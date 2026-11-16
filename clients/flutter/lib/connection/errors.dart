typedef Json = Map<String, dynamic>;

class ApiFailure implements Exception {
  const ApiFailure(this.code, [this.status = 0]);
  final String code;
  final int status;
  bool get denied => status == 401 || status == 403;
  @override
  String toString() => 'ApiFailure($code, status: $status)';
}

Json object(Object? value) {
  if (value is! Json) throw const ApiFailure('invalid_response');
  return value;
}
