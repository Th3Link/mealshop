import 'dart:async';
import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter/foundation.dart';
import 'package:http/http.dart' as http;

typedef Json = Map<String, dynamic>;

class ApiFailure implements Exception {
  const ApiFailure(this.code, [this.status = 0]);
  final String code;
  final int status;
  bool get denied => status == 401 || status == 403;
}

Json object(Object? value) {
  if (value is! Json) throw const ApiFailure('invalid_response');
  return value;
}

/// A configured installation root, never a response-controlled redirect target.
class Server {
  Server(String value, {bool allowLoopback = kDebugMode}) {
    String decoded;
    try {
      decoded = Uri.decodeComponent(value.trim());
    } on FormatException {
      throw const ApiFailure('invalid_server');
    }
    if (RegExp(r'/\.{1,2}(?:/|$)').hasMatch(decoded) ||
        decoded.runes.any((c) => c < 32)) {
      throw const ApiFailure('invalid_server');
    }
    final parsed = Uri.tryParse(value.trim());
    if (parsed == null ||
        !parsed.hasAuthority ||
        parsed.host.isEmpty ||
        parsed.userInfo.isNotEmpty ||
        parsed.hasQuery ||
        parsed.hasFragment ||
        !(parsed.scheme == 'https' ||
            (allowLoopback &&
                parsed.scheme == 'http' &&
                ['127.0.0.1', 'localhost', '::1'].contains(parsed.host)))) {
      throw const ApiFailure('invalid_server');
    }
    for (final segment in parsed.pathSegments) {
      if (segment == '.' ||
          segment == '..' ||
          segment.contains('/') ||
          segment.contains(r'\') ||
          segment.runes.any((c) => c < 32)) {
        throw const ApiFailure('invalid_server');
      }
    }
    root = parsed.replace(path: parsed.path.replaceAll(RegExp(r'/+$'), ''));
  }
  late final Uri root;
  Uri path(String suffix) => Uri.parse('$root$suffix');
  Uri validate(String value) {
    final candidate = Uri.tryParse(value);
    if (candidate == null ||
        candidate.origin != root.origin ||
        candidate.userInfo.isNotEmpty ||
        candidate.hasFragment ||
        !candidate.path.startsWith('${root.path}/')) {
      throw const ApiFailure('untrusted_login');
    }
    return candidate;
  }
}

class Credential {
  const Credential(this.server, this.user, this.password);
  final Server server;
  final String user;
  final String password;
  Json toJson() => {
    'server': server.root.toString(),
    'user': user,
    'password': password,
  };
  static Credential parse(Json value) => Credential(
    Server(value['server'] as String),
    value['user'] as String,
    value['password'] as String,
  );
}

class LoginFlow {
  LoginFlow(this.server, this.login, this.endpoint, this.token);
  final Server server;
  final Uri login;
  final Uri endpoint;
  final String token;
}

class Nextcloud {
  Nextcloud({http.Client? client}) : _client = client ?? http.Client();
  final http.Client _client;
  void close() => _client.close();

  Future<Object?> request(
    Uri uri, {
    String method = 'GET',
    Object? body,
    Credential? credential,
    bool form = false,
    bool polling = false,
  }) async {
    final request = http.Request(method, uri)..followRedirects = false;
    request.headers['accept'] = 'application/json';
    request.headers['user-agent'] = 'Mealshop Android/0.2';
    if (credential != null) {
      credential.server.validate(uri.toString());
      request.headers['authorization'] =
          'Basic ${base64Encode(utf8.encode('${credential.user}:${credential.password}'))}';
      request.headers['x-mealshop-request'] = '1';
      request.headers['OCS-APIRequest'] = 'true';
    }
    if (form) {
      request.bodyFields = Map<String, String>.from(body! as Map);
    } else if (body != null) {
      request.headers['content-type'] = 'application/json';
      request.body = jsonEncode(body);
    }
    try {
      return await (() async {
        final response = await _client.send(request);
        final bytes = <int>[];
        await for (final chunk in response.stream) {
          if (bytes.length + chunk.length > 4194304) {
            throw const ApiFailure('response_too_large');
          }
          bytes.addAll(chunk);
        }
        if (polling && response.statusCode == 404) return null;
        Object? value;
        try {
          value = jsonDecode(utf8.decode(bytes));
        } on FormatException {
          throw ApiFailure('invalid_response', response.statusCode);
        }
        if (response.statusCode < 200 || response.statusCode >= 300) {
          throw ApiFailure(
            value is Json
                ? value['code'] as String? ?? 'request_failed'
                : 'request_failed',
            response.statusCode,
          );
        }
        return value;
      })().timeout(const Duration(seconds: 15));
    } on ApiFailure {
      rethrow;
    } catch (_) {
      throw const ApiFailure('connection_failed');
    }
  }

  Future<Uint8List> bytes(Credential who, String path) async {
    final uri = who.server.path('/index.php/apps/app_api/proxy/mealshop$path');
    who.server.validate(uri.toString());
    final request = http.Request('GET', uri)..followRedirects = false;
    request.headers['authorization'] =
        'Basic ${base64Encode(utf8.encode('${who.user}:${who.password}'))}';
    request.headers['x-mealshop-request'] = '1';
    return _binary(request);
  }

  Future<Uint8List> _binary(http.Request request) async {
    try {
      return await (() async {
        final response = await _client.send(request);
        if (response.statusCode != 200) {
          throw ApiFailure('request_failed', response.statusCode);
        }
        final bytes = BytesBuilder(copy: false);
        await for (final chunk in response.stream) {
          if (bytes.length + chunk.length > 5242880) {
            throw const ApiFailure('response_too_large');
          }
          bytes.add(chunk);
        }
        return bytes.takeBytes();
      })().timeout(const Duration(seconds: 20));
    } on ApiFailure {
      rethrow;
    } catch (_) {
      throw const ApiFailure('connection_failed');
    }
  }

  Future<Json> upload(
    Credential who,
    String path,
    Uint8List bytes,
    String mime,
  ) async {
    if (bytes.length > 5242880 ||
        !['image/jpeg', 'image/png', 'image/webp'].contains(mime)) {
      throw const ApiFailure('image_invalid');
    }
    final uri = who.server.path('/index.php/apps/app_api/proxy/mealshop$path');
    who.server.validate(uri.toString());
    final request = http.Request('POST', uri)..followRedirects = false;
    request.headers.addAll({
      'authorization':
          'Basic ${base64Encode(utf8.encode('${who.user}:${who.password}'))}',
      'x-mealshop-request': '1',
      'content-type': mime,
    });
    request.bodyBytes = bytes;
    try {
      return await (() async {
        final response = await _client.send(request);
        final data = BytesBuilder(copy: false);
        await for (final chunk in response.stream) {
          if (data.length + chunk.length > 65536) {
            throw const ApiFailure('response_too_large');
          }
          data.add(chunk);
        }
        final result = object(jsonDecode(utf8.decode(data.takeBytes())));
        if (response.statusCode < 200 || response.statusCode >= 300) {
          throw ApiFailure(
            result['code'] as String? ?? 'image_invalid',
            response.statusCode,
          );
        }
        return result;
      })().timeout(const Duration(seconds: 20));
    } on ApiFailure {
      rethrow;
    } catch (_) {
      throw const ApiFailure('connection_failed');
    }
  }

  Future<LoginFlow> start(Server server) async {
    final value = object(
      await request(server.path('/index.php/login/v2'), method: 'POST'),
    );
    final poll = object(value['poll']);
    return LoginFlow(
      server,
      server.validate(value['login'] as String),
      server.validate(poll['endpoint'] as String),
      poll['token'] as String,
    );
  }

  Future<Credential?> poll(LoginFlow flow) async {
    final result = await request(
      flow.endpoint,
      method: 'POST',
      body: {'token': flow.token},
      form: true,
      polling: true,
    );
    if (result == null) return null;
    final value = object(result);
    final canonical = Server(value['server'] as String);
    if (canonical.root != flow.server.root) {
      throw const ApiFailure('untrusted_login');
    }
    final user = value['loginName'] as String;
    final password = value['appPassword'] as String;
    if (user.isEmpty || user.contains(':') || password.isEmpty) {
      throw const ApiFailure('invalid_response');
    }
    return Credential(canonical, user, password);
  }

  Future<Json> api(
    Credential who,
    String path, {
    String method = 'GET',
    Object? body,
  }) async => object(
    await request(
      who.server.path('/index.php/apps/app_api/proxy/mealshop$path'),
      method: method,
      body: body,
      credential: who,
    ),
  );
}
