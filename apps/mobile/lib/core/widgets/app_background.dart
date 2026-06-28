import 'package:flutter/material.dart';
import 'package:veyra_mobile/core/theme/app_theme.dart';

/// Which ambient treatment a screen uses, mirroring the Claude Design folder:
/// - [app]: amber glow top-right + large V monogram + bottom arc lines
///   (`home-ios`, `vehicle-detail`, `reminders`, `documents`, states).
/// - [auth]: amber glow top-center + V monogram + arcs (`login`, `register`).
/// - [sheet]: glow top-right only — the bottom sheet covers the lower texture
///   (`add-fuel-sheet`).
enum AmbientVariant { app, auth, sheet }

/// Shared screen backdrop, ported 1:1 from the Claude Design ambient layer
/// (`.veyra-amb` / `.glow` + `.veyra-tex`). Painted in the design's 390x844
/// reference frame and scaled to fill. Decorative + non-interactive.
class AppBackground extends StatelessWidget {
  const AppBackground({
    required this.child,
    this.variant = AmbientVariant.app,
    super.key,
  });

  final Widget child;
  final AmbientVariant variant;

  @override
  Widget build(BuildContext context) => Stack(
        children: [
          const Positioned.fill(child: ColoredBox(color: VeyraColors.bg)),
          Positioned.fill(
            child: IgnorePointer(
              child: CustomPaint(painter: _AmbientPainter(variant)),
            ),
          ),
          child,
        ],
      );
}

class _AmbientPainter extends CustomPainter {
  _AmbientPainter(this.variant);

  final AmbientVariant variant;

  // Design reference frame (iPhone logical points). All coordinates below are
  // expressed in this space, then scaled to the actual canvas size.
  static const _refWidth = 390.0;
  static const _refHeight = 844.0;

  static const _amber = Color(0xFFF26A21);
  static const _ink = Color(0xFFE6EAF0);

  @override
  void paint(Canvas canvas, Size size) {
    canvas
      ..save()
      ..scale(size.width / _refWidth, size.height / _refHeight);

    if (variant == AmbientVariant.auth) {
      _paintAuthGlow(canvas);
    } else {
      _paintAppGlow(canvas);
    }
    if (variant != AmbientVariant.sheet) {
      _paintMonogram(canvas);
      _paintArcs(canvas);
    }

    canvas.restore();
  }

  // App glow: amber ellipse top-right (cx372 cy70 rx220 ry200); radial stops
  // 0.16 -> 0.05 @55% -> 0.
  void _paintAppGlow(Canvas canvas) {
    final rect = Rect.fromCenter(
      center: const Offset(372, 70),
      width: 440,
      height: 400,
    );
    canvas.drawOval(
      rect,
      Paint()
        ..shader = RadialGradient(
          colors: [
            _amber.withValues(alpha: 0.16),
            _amber.withValues(alpha: 0.05),
            _amber.withValues(alpha: 0),
          ],
          stops: const [0, 0.55, 1],
        ).createShader(rect),
    );
  }

  // Auth glow: amber circle top-center (cx195 cy60 r180); radial 0.22 -> 0 @70%.
  void _paintAuthGlow(Canvas canvas) {
    final rect = Rect.fromCircle(center: const Offset(195, 60), radius: 180);
    canvas.drawRect(
      rect,
      Paint()
        ..shader = RadialGradient(
          colors: [
            _amber.withValues(alpha: 0.22),
            _amber.withValues(alpha: 0),
          ],
          stops: const [0, 0.7],
        ).createShader(rect),
    );
  }

  // One large faint V monogram at translate(58,470) scale(2.35). Stroke width
  // 2.4 is pre-scale, so it thickens with the group exactly like the SVG.
  void _paintMonogram(Canvas canvas) {
    final outer = Path()
      ..moveTo(14, 22)
      ..lineTo(41, 22)
      ..lineTo(60, 68)
      ..lineTo(79, 22)
      ..lineTo(106, 22)
      ..lineTo(60, 106)
      ..close();
    final fold = Path()
      ..moveTo(41, 22)
      ..lineTo(60, 68)
      ..lineTo(60, 106);
    Paint stroke(double alpha) => Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 2.4
      ..strokeJoin = StrokeJoin.round
      ..color = _ink.withValues(alpha: alpha);
    canvas
      ..save()
      ..translate(58, 470)
      ..scale(2.35)
      ..drawPath(outer, stroke(0.05))
      ..drawPath(fold, stroke(0.03))
      ..restore();
  }

  // Three concentric arc lines near the bottom (SVG sweep-flag 1 -> clockwise).
  void _paintArcs(Canvas canvas) {
    final paint = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.4
      ..strokeCap = StrokeCap.round
      ..color = _ink.withValues(alpha: 0.045);
    Path arc(double y, double radius, double endX) => Path()
      ..moveTo(-40, y)
      ..arcToPoint(
        Offset(endX, y),
        radius: Radius.circular(radius),
      );
    canvas
      ..drawPath(arc(690, 150, 250), paint)
      ..drawPath(arc(740, 200, 300), paint)
      ..drawPath(arc(792, 250, 348), paint);
  }

  @override
  bool shouldRepaint(covariant _AmbientPainter oldDelegate) =>
      oldDelegate.variant != variant;
}
