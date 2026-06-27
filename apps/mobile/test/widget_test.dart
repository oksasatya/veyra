import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:veyra_mobile/core/widgets/veyra_mark.dart';

void main() {
  testWidgets('VeyraMark renders', (tester) async {
    await tester.pumpWidget(
      const MaterialApp(home: Scaffold(body: VeyraMark())),
    );
    expect(find.byType(VeyraMark), findsOneWidget);
  });
}
