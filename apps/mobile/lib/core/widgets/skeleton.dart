import 'dart:async';

import 'package:flutter/material.dart';
import 'package:veyra_mobile/core/theme/app_theme.dart';

/// A shimmering placeholder block (design `.sk`). A pale highlight sweeps across
/// a surface-2 base; the sweep is suppressed under `prefers-reduced-motion`.
class SkeletonBox extends StatefulWidget {
  const SkeletonBox({
    required this.height,
    this.width,
    this.radius = 8,
    super.key,
  });

  final double? width;
  final double height;
  final double radius;

  @override
  State<SkeletonBox> createState() => _SkeletonBoxState();
}

class _SkeletonBoxState extends State<SkeletonBox>
    with SingleTickerProviderStateMixin {
  late final AnimationController _controller = AnimationController(
    vsync: this,
    duration: const Duration(milliseconds: 1300),
  );

  @override
  void initState() {
    super.initState();
    unawaited(_controller.repeat());
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final reduceMotion = MediaQuery.maybeOf(context)?.disableAnimations ?? false;
    final base = ClipRRect(
      borderRadius: BorderRadius.circular(widget.radius),
      child: SizedBox(
        width: widget.width,
        height: widget.height,
        child: const ColoredBox(color: VeyraColors.surface2),
      ),
    );
    if (reduceMotion) {
      return Opacity(opacity: 0.9, child: base);
    }
    return ClipRRect(
      borderRadius: BorderRadius.circular(widget.radius),
      child: SizedBox(
        width: widget.width,
        height: widget.height,
        child: AnimatedBuilder(
          animation: _controller,
          builder: (context, _) => DecoratedBox(
            decoration: const BoxDecoration(color: VeyraColors.surface2),
            child: FractionallySizedBox(
              widthFactor: 1,
              child: _Sweep(progress: _controller.value),
            ),
          ),
        ),
      ),
    );
  }
}

class _Sweep extends StatelessWidget {
  const _Sweep({required this.progress});

  final double progress;

  @override
  Widget build(BuildContext context) => ShaderMask(
        blendMode: BlendMode.srcOver,
        shaderCallback: (rect) {
          final dx = rect.width * (progress * 2 - 1);
          return LinearGradient(
            colors: [
              VeyraColors.surface2.withValues(alpha: 0),
              VeyraColors.text.withValues(alpha: 0.08),
              VeyraColors.surface2.withValues(alpha: 0),
            ],
          ).createShader(
            Rect.fromLTWH(dx, 0, rect.width, rect.height),
          );
        },
        child: const SizedBox.expand(),
      );
}
