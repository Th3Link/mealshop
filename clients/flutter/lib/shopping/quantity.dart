/// Exact six-decimal input; presentation never round-trips through floating point.
class Quantity {
  const Quantity(this.original, this.amount, this.unit);
  final String original;
  final String amount;
  final String unit;
  static Quantity parse(String input, String unit, String language) {
    if (!['g', 'kg', 'ml', 'l', 'piece'].contains(unit)) {
      throw const FormatException('unit');
    }
    final original = input.trim();
    final separator = language == 'de' ? ',' : '.';
    if (!RegExp('^[0-9]{1,12}(?:${RegExp.escape(separator)}[0-9]{1,6})?\$')
        .hasMatch(original)) {
      throw const FormatException('quantity');
    }
    final parts = original.split(separator);
    final whole = BigInt.parse(parts.first);
    final fraction = parts.length == 1 ? '' : parts[1];
    final atoms =
        whole * BigInt.from(1000000) + BigInt.parse(fraction.padRight(6, '0'));
    final factor = ['kg', 'l'].contains(unit) ? BigInt.from(1000) : BigInt.one;
    if (atoms * factor > BigInt.from(1000000000000000)) {
      throw const FormatException('range');
    }
    final canonical = original.replaceAll(separator, '.');
    if (whole == BigInt.zero &&
        BigInt.parse(fraction.padRight(6, '0')) == BigInt.zero) {
      throw const FormatException('positive');
    }
    return Quantity(original, canonical, unit);
  }

  Map<String, Object> toJson() => {'amount': amount, 'unit': unit};
  static String display(String canonical, String language) {
    if (!RegExp(r'^\d+(?:\.\d{1,6})?$').hasMatch(canonical)) {
      throw const FormatException('quantity');
    }
    final value = canonical.contains('.')
        ? canonical
              .replaceFirst(RegExp(r'0+$'), '')
              .replaceFirst(RegExp(r'\.$'), '')
        : canonical;
    return language == 'de' ? value.replaceAll('.', ',') : value;
  }

  static bool positive(String value) =>
      BigInt.parse(value.replaceAll('.', '')) > BigInt.zero;
}
