import 'dart:async';

import 'package:flutter/material.dart';
import 'package:veyra_mobile/core/theme/app_theme.dart';
import 'package:veyra_mobile/core/widgets/app_background.dart';
import 'package:veyra_mobile/core/widgets/veyra_mark.dart';

const _wordmark = 'veyra';
const _logoSize = 52.0;
const _lockupGap = 14.0;

/// First frame shown while the saved session is being restored. The V monogram
/// animates in at screen-centre (matching the static native launch image), then
/// slides left as the "veyra" wordmark types itself in beside it (no caret),
/// landing as a centred lockup. Router redirects away once auth resolves.
/// Honors reduced-motion.
class SplashScreen extends StatefulWidget {
  const SplashScreen({super.key});

  @override
  State<SplashScreen> createState() => _SplashScreenState();
}

class _SplashScreenState extends State<SplashScreen>
    with SingleTickerProviderStateMixin {
  late final AnimationController _controller = AnimationController(
    vsync: this,
    duration: const Duration(milliseconds: 1050),
  );

  // The logo settles first; then the wordmark types in, letter by letter.
  late final Animation<double> _logoFade = _curve(0, 0.4);
  late final Animation<double> _logoScale = Tween<double>(begin: 0.85, end: 1)
      .animate(_curve(0, 0.46, Curves.easeOutCubic));
  late final Animation<double> _type = _curve(0.46, 1);

  bool _started = false;

  CurvedAnimation _curve(double begin, double end,
          [Curve curve = Curves.easeOut]) =>
      CurvedAnimation(
        parent: _controller,
        curve: Interval(begin, end, curve: curve),
      );

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    if (_started) return;
    _started = true;
    // Reduced-motion users get the final frame with no animation.
    if (MediaQuery.maybeOf(context)?.disableAnimations ?? false) {
      _controller.value = 1;
    } else {
      unawaited(_controller.forward());
    }
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  double _wordmarkWidth(TextStyle style) {
    final painter = TextPainter(
      text: TextSpan(text: _wordmark, style: style),
      textDirection: TextDirection.ltr,
    )..layout();
    return painter.width;
  }

  @override
  Widget build(BuildContext context) {
    final wordStyle = soraDisplay(size: 36);
    // How far the lockup must shift so the logo starts dead-centre and ends as
    // a centred [logo + gap + wordmark] group.
    final slideStart = (_wordmarkWidth(wordStyle) + _lockupGap) / 2;
    return Scaffold(
      body: AppBackground(
        variant: AmbientVariant.auth,
        child: Center(
          child: AnimatedBuilder(
            animation: _controller,
            builder: (context, _) {
              final shown = (_type.value * _wordmark.length)
                  .round()
                  .clamp(0, _wordmark.length);
              return Transform.translate(
                offset: Offset((1 - _type.value) * slideStart, 0),
                child: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Opacity(
                      opacity: _logoFade.value,
                      child: Transform.scale(
                        scale: _logoScale.value,
                        child: const VeyraMark(size: _logoSize),
                      ),
                    ),
                    const SizedBox(width: _lockupGap),
                    // Zero-opacity full word reserves the final width so the
                    // visible text grows in place without nudging the logo.
                    Stack(
                      children: [
                        Opacity(
                          opacity: 0,
                          child: Text(_wordmark, style: wordStyle),
                        ),
                        Text(_wordmark.substring(0, shown), style: wordStyle),
                      ],
                    ),
                  ],
                ),
              );
            },
          ),
        ),
      ),
    );
  }
}
