import 'package:flutter_test/flutter_test.dart';
import 'package:mealshop/shopping/quantity.dart';
import 'package:mealshop/ui/products.dart';

void main() {
  test('German six-place input retains every digit and its original text', () {
    final q = Quantity.parse('200,000001', 'g', 'de');
    expect(q.amount, '200.000001');
    expect(q.original, '200,000001');
    expect(q.unit, 'g');
    expect(Quantity.display(q.amount, 'de'), '200,000001');
    expect(Quantity.display('200.000000', 'de'), '200');
    expect(Quantity.parse('0.000001', 'ml', 'en').amount, '0.000001');
    expect(
      Quantity.parse('999999999.123456', 'piece', 'en').amount,
      '999999999.123456',
    );
  });
  test('ambiguous grouping, wrong locale, zero, exponent and excess precision are rejected', () {
    for (final input in [
      '0',
      '-1',
      '1.000,5',
      '1,000.5',
      '1.5',
      '1,0000001',
      '1e3',
      'NaN',
      '',
    ]) {
      expect(
        () => Quantity.parse(input, 'g', 'de'),
        throwsFormatException,
        reason: input,
      );
    }
    expect(() => Quantity.parse('1', 'cup', 'en'), throwsFormatException);
    expect(
      () => Quantity.parse('1000000000.000001', 'g', 'en'),
      throwsFormatException,
    );
    expect(() => Quantity.parse('1000001', 'kg', 'en'), throwsFormatException);
    expect(Quantity.parse('001,2300', 'g', 'de').amount, '001.2300');
  });
  test('GTIN input validates the checksum without inventing identity', () {
    expect(validGtin('4006381333931'), isTrue);
    expect(validGtin('4006381333932'), isFalse);
    expect(validGtin('00000000'), isFalse);
    expect(validGtin('abc'), isFalse);
  });
}
