import 'package:flutter/material.dart';
import 'package:veyra_mobile/core/theme/app_theme.dart';

/// The Veyra V monogram, painted from the brand SVG path.
class VeyraMark extends StatelessWidget {
  const VeyraMark({super.key, this.size = 40, this.color = VeyraColors.accent});
  final double size;
  final Color color;

  @override
  Widget build(BuildContext context) =>
      CustomPaint(size: Size(size, size), painter: _MarkPainter(color));
}

class _MarkPainter extends CustomPainter {
  _MarkPainter(this.color);
  final Color color;

  @override
  void paint(Canvas canvas, Size size) {
    final s = size.width / 120.0;
    Offset p(double x, double y) => Offset(x * s, y * s);
    final path = Path()
      ..moveTo(p(14, 22).dx, p(14, 22).dy)
      ..lineTo(p(41, 22).dx, p(41, 22).dy)
      ..lineTo(p(60, 68).dx, p(60, 68).dy)
      ..lineTo(p(79, 22).dx, p(79, 22).dy)
      ..lineTo(p(106, 22).dx, p(106, 22).dy)
      ..lineTo(p(60, 106).dx, p(60, 106).dy)
      ..close();
    canvas.drawPath(
      path,
      Paint()
        ..color = color
        ..style = PaintingStyle.fill
        ..isAntiAlias = true,
    );
  }

  @override
  bool shouldRepaint(_MarkPainter oldDelegate) => oldDelegate.color != color;
}
