"""Bridge boundary tests use a fresh loopback fixture, never a user's Nextcloud instance."""
import base64
import importlib.util
import io
import json
import pathlib
import threading
import unittest
import urllib.request
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
spec=importlib.util.spec_from_file_location('mealshop_mcp_stdio',pathlib.Path(__file__).resolve().parents[1]/'scripts/mcp_stdio.py')
bridge=importlib.util.module_from_spec(spec);spec.loader.exec_module(bridge)
sender_spec=importlib.util.spec_from_file_location('mealshop_suggest_recipe',pathlib.Path(__file__).resolve().parents[1]/'scripts/suggest_recipe.py')
sender=importlib.util.module_from_spec(sender_spec);sender_spec.loader.exec_module(sender)


class Fixture:
    def __init__(self, status=200, response=None, redirect=None):
        self.requests=[]
        outer=self
        class Handler(BaseHTTPRequestHandler):
            def log_message(self,*args):pass
            def do_POST(self):
                body=self.rfile.read(int(self.headers.get('Content-Length','0')))
                outer.requests.append((dict(self.headers),body))
                self.send_response(status)
                if redirect:self.send_header('Location',redirect)
                self.end_headers()
                self.wfile.write(response if response is not None else json.dumps({'jsonrpc':'2.0','id':1,'result':{'ok':True}}).encode())
        self.server=ThreadingHTTPServer(('127.0.0.1',0),Handler)
        self.thread=threading.Thread(target=self.server.serve_forever,daemon=True);self.thread.start()
        self.url=f'http://127.0.0.1:{self.server.server_port}/api/household/mcp'
    def close(self):self.server.shutdown();self.server.server_close();self.thread.join()
    def env(self):return {'MEALSHOP_MCP_URL':self.url,'MEALSHOP_MCP_USER':'synthetic','MEALSHOP_MCP_APP_PASSWORD':'synthetic-only-password'}


class StdioTests(unittest.TestCase):
    def fixture(self,**kwargs):
        f=Fixture(**kwargs);self.addCleanup(f.close);return f
    def test_configuration_rejects_nonlocal_plaintext_credentials_in_url_and_query(self):
        env=self.fixture().env()
        for url in ['http://example.org/api/household/mcp','https://user:pass@example.org/api/household/mcp','https://example.org/api/household/mcp?token=private','https://example.org/api/household/mcp#fragment']:
            with self.assertRaises(ValueError):bridge.configuration({**env,'MEALSHOP_MCP_URL':url})
        self.assertTrue(bridge.configuration({**env,'MEALSHOP_MCP_URL':'https://example.org/cloud/index.php/apps/app_api/proxy/mealshop/api/households/home-1/domain/api/household/mcp'})[0].startswith('https://'))
    def test_modern_stdio_preserves_request_and_mirrors_metadata_without_returning_credentials(self):
        f=self.fixture();out=io.StringIO();message={'jsonrpc':'2.0','id':1,'method':'tools/call','params':{'name':'recipe_search','arguments':{},'_meta':{'io.modelcontextprotocol/protocolVersion':'2026-07-28','io.modelcontextprotocol/clientCapabilities':{}}}}
        self.assertEqual(bridge.run(io.BytesIO((json.dumps(message)+'\n').encode()),out,f.env()),0)
        reply=json.loads(out.getvalue());self.assertTrue(reply['result']['ok']);headers,body=f.requests[0]
        headers={key.lower():value for key,value in headers.items()}
        self.assertEqual(headers['mcp-method'],'tools/call');self.assertEqual(headers['mcp-name'],'recipe_search');self.assertEqual(headers['x-mealshop-request'],'1')
        self.assertTrue(headers['authorization'].startswith('Basic '));self.assertEqual(json.loads(body),message)
        self.assertNotIn('synthetic-only-password',out.getvalue());self.assertNotIn(headers['authorization'],out.getvalue())
    def test_redirect_never_forwards_app_password_to_another_destination(self):
        destination=self.fixture();origin=self.fixture(status=302,redirect=destination.url)
        result,_=bridge.forward(urllib.request.build_opener(bridge.NoRedirect),origin.url,base64.b64encode(b'synthetic:synthetic-only-password').decode(),{'jsonrpc':'2.0','id':1,'method':'tools/list','params':{}},bridge.LEGACY)
        self.assertIn('error',result);self.assertEqual(destination.requests,[])
    def test_host_html_and_mismatched_responses_are_not_republished(self):
        for response in [b'<html>private upstream error</html>',b'{"jsonrpc":"2.0","id":999,"result":{"private":"other request"}}']:
            f=self.fixture(response=response);out=io.StringIO();bridge.run(io.BytesIO(b'{"jsonrpc":"2.0","id":1,"method":"tools/list"}\n'),out,f.env())
            result=json.loads(out.getvalue());self.assertEqual(result['id'],1);self.assertIn('error',result);self.assertNotIn('private',out.getvalue())
    def test_malformed_and_oversized_lines_produce_only_bounded_json_rpc_errors(self):
        f=self.fixture();out=io.StringIO();source=io.BytesIO(b'{\n'+b'x'*(bridge.LIMIT+1)+b'\n')
        bridge.run(source,out,f.env());values=[json.loads(line) for line in out.getvalue().splitlines()]
        self.assertEqual([v['error']['code'] for v in values],[-32700,-32600]);self.assertEqual(f.requests,[])

    def test_suggestion_sender_reports_each_line_and_never_prints_credentials(self):
        reply={'jsonrpc':'2.0','id':1,'result':{'isError':False,'structuredContent':{'suggestion_id':'suggestion-'+'a'*32,'replayed':False}}}
        f=self.fixture(response=json.dumps(reply).encode());out=io.StringIO()
        lines=json.dumps({'title':'Synthetic','ingredients':['2 tbsp oil'],'source_url':'https://example.org/r','note':None}).encode()+b'\n\n{"unknown":1}\nnot json\n'
        self.assertEqual(sender.run(io.BytesIO(lines),out,f.env()),1)
        rows=[row.split('\t') for row in out.getvalue().splitlines()]
        self.assertEqual([row[:2] for row in rows],[['1','added'],['3','failed'],['4','failed']]);self.assertEqual(rows[0][2],'suggestion-'+'a'*32)
        self.assertEqual(len(f.requests),1);sent=json.loads(f.requests[0][1])
        self.assertEqual(sent['params']['name'],'recipe_suggest');self.assertEqual(sent['params']['arguments'],{'title':'Synthetic','ingredients':['2 tbsp oil'],'source_url':'https://example.org/r'})
        self.assertNotIn('synthetic-only-password',out.getvalue())
    def test_suggestion_sender_surfaces_tool_errors_as_codes(self):
        reply={'jsonrpc':'2.0','id':1,'result':{'isError':True,'structuredContent':{'code':'suggestion_capacity','message':'x'}}}
        f=self.fixture(response=json.dumps(reply).encode());out=io.StringIO()
        self.assertEqual(sender.run(io.BytesIO(b'{"title":"Synthetic"}\n'),out,f.env()),1)
        self.assertEqual(out.getvalue(),'1\tfailed\tsuggestion_capacity\n')


if __name__=='__main__':unittest.main()
