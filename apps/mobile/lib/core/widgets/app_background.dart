import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:veyra_mobile/core/theme/app_theme.dart';

/// Shared screen backdrop — carbon base plus a painted Veyra texture: a soft
/// amber glow at the top, faint concentric gauge arcs, and a large low-opacity
/// V monogram. Decorative + non-interactive; sits behind every screen's content.
class AppBackground extends StatelessWidget {
  const AppBackground({required this.child, super.key});
  final Widget child;

  @override
  Widget build(BuildContext context) => Stack(
        children: [
          const Positioned.fill(child: ColoredBox(color: VeyraColors.bg)),
          Positioned.fill(
            child: IgnorePointer(
              child: CustomPaint(painter: _BackdropPainter()),
            ),
          ),
          child,
        ],
      );
}

class _BackdropPainter extends CustomPainter {
  static const _amber = Color(0xFFF26A21);
  static const _teal = Color(0xFF34D1C4);

  @override
  void paint(Canvas canvas, Size size) {
    final w = size.width;
    final h = size.height;

    // 1. Ambient amber glow at the top.
    final glowRect = Rect.fromCircle(center: Offset(w * 0.5, -h * 0.04), radius: w);
    canvas.drawRect(
      Offset.zero & size,
      Paint()
        ..shader = RadialGradient(
          radius: 0.6,
          colors: [_amber.withValues(alpha: 0.20), _amber.withValues(alpha: 0)],
        ).createShader(glowRect),
    );

    // 2. Cool wash, lower-left.
    final washRect = Rect.fromCircle(center: Offset(0, h * 0.92), radius: w * 0.9);
    canvas.drawRect(
      Offset.zero & size,
      Paint()
        ..shader = RadialGradient(
          radius: 0.5,
          colors: [_teal.withValues(alpha: 0.07), _teal.withValues(alpha: 0)],
        ).createShader(washRect),
    );

    // 3. Concentric gauge arcs sweeping from the top-right (speedometer motif).
    final arcCenter = Offset(w * 0.92, h * 0.18);
    final arcPaint = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.4
      ..color = _amber.withValues(alpha: 0.07);
    for (var i = 0; i < 4; i++) {
      final r = w * (0.42 + i * 0.17);
      canvas.drawArc(
        Rect.fromCircle(center: arcCenter, radius: r),
        math.pi * 0.55,
        math.pi * 0.72,
        false,
        arcPaint,
      );
    }

    // 4. Large faint V monogram texture, lower area.
    final s = (w * 0.92) / 120.0;
    final dx = w * 0.16;
    final dy = h * 0.60;
    Offset p(double x, double y) => Offset(dx + x * s, dy + y * s);
    final v = Path()
      ..moveTo(p(14, 22).dx, p(14, 22).dy)
      ..lineTo(p(41, 22).dx, p(41, 22).dy)
      ..lineTo(p(60, 68).dx, p(60, 68).dy)
      ..lineTo(p(79, 22).dx, p(79, 22).dy)
      ..lineTo(p(106, 22).dx, p(106, 22).dy)
      ..lineTo(p(60, 106).dx, p(60, 106).dy)
      ..close();
    canvas.drawPath(v, Paint()..color = _amber.withValues(alpha: 0.05));
  }

  @override
  bool shouldRepaint(covariant _BackdropPainter oldDelegate) => false;
}
