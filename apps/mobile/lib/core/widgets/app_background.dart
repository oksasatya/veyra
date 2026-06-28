import 'package:flutter/material.dart';
import 'package:veyra_mobile/core/theme/app_theme.dart';
import 'package:veyra_mobile/core/widgets/veyra_mark.dart';

/// Shared screen backdrop: the carbon base + a soft ambient amber glow at the
/// top and a faint Veyra-mark watermark bottom-right. Gives screens depth
/// instead of a flat fill. Decorative + non-interactive.
class AppBackground extends StatelessWidget {
  const AppBackground({required this.child, super.key});
  final Widget child;

  @override
  Widget build(BuildContext context) => Stack(
    children: [
      const Positioned.fill(child: ColoredBox(color: VeyraColors.bg)),
      // Top ambient glow.
      const Positioned(
        top: -180,
        left: -80,
        right: -80,
        height: 420,
        child: IgnorePointer(
          child: DecoratedBox(
            decoration: BoxDecoration(
              gradient: RadialGradient(
                radius: 0.8,
                colors: [Color(0x33F26A21), Color(0x00F26A21)],
              ),
            ),
          ),
        ),
      ),
      // Secondary cool wash, bottom-left, very subtle.
      const Positioned(
        bottom: -220,
        left: -140,
        height: 420,
        width: 420,
        child: IgnorePointer(
          child: DecoratedBox(
            decoration: BoxDecoration(
              gradient: RadialGradient(
                radius: 0.7,
                colors: [Color(0x1434D1C4), Color(0x0034D1C4)],
              ),
            ),
          ),
        ),
      ),
      // Faint brand watermark.
      const Positioned(
        bottom: -56,
        right: -56,
        child: IgnorePointer(
          child: Opacity(
            opacity: 0.05,
            child: VeyraMark(size: 280),
          ),
        ),
      ),
      child,
    ],
  );
}
