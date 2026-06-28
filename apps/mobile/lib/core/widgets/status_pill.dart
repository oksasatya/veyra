import 'package:flutter/material.dart';
import 'package:veyra_mobile/core/theme/app_theme.dart';

/// Semantic tone for a [StatusPill], mapped to the design's pill variants
/// (`.pill.over/.soon/.up/.ok/.none`). Each tone carries a tinted background,
/// a solid foreground, and a matching border — never a bare colored chip.
enum PillTone { danger, accent, ok, muted }

/// A small rounded status chip used across reminders and documents.
class StatusPill extends StatelessWidget {
  const StatusPill(this.label, {required this.tone, super.key});

  final String label;
  final PillTone tone;

  Color get _base => switch (tone) {
        PillTone.danger => VeyraColors.danger,
        PillTone.accent => VeyraColors.accent,
        PillTone.ok => VeyraColors.ok,
        PillTone.muted => VeyraColors.textMuted,
      };

  @override
  Widget build(BuildContext context) {
    final muted = tone == PillTone.muted;
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 5),
      decoration: BoxDecoration(
        color: muted ? VeyraColors.surface2 : _base.withValues(alpha: 0.14),
        borderRadius: BorderRadius.circular(999),
        border: Border.all(
          color: muted ? VeyraColors.border : _base.withValues(alpha: 0.3),
        ),
      ),
      child: Text(
        label,
        style: TextStyle(
          color: _base,
          fontSize: 12,
          fontWeight: FontWeight.w600,
        ),
      ),
    );
  }
}
