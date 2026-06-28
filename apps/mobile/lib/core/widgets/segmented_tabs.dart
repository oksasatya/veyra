import 'package:flutter/material.dart';
import 'package:veyra_mobile/core/theme/app_theme.dart';

/// Horizontal pill segmented control (design `.tabs > .seg`). Active segment is
/// solid amber on ink; inactive segments are surface chips with muted labels.
class SegmentedTabs extends StatelessWidget {
  const SegmentedTabs({
    required this.labels,
    required this.index,
    required this.onChanged,
    super.key,
  });

  final List<String> labels;
  final int index;
  final ValueChanged<int> onChanged;

  @override
  Widget build(BuildContext context) => SingleChildScrollView(
        scrollDirection: Axis.horizontal,
        padding: const EdgeInsets.symmetric(horizontal: 20),
        child: Row(
          children: [
            for (var i = 0; i < labels.length; i++)
              Padding(
                padding: EdgeInsets.only(right: i == labels.length - 1 ? 0 : 8),
                child: _Seg(
                  label: labels[i],
                  active: i == index,
                  onTap: () => onChanged(i),
                ),
              ),
          ],
        ),
      );
}

class _Seg extends StatelessWidget {
  const _Seg({required this.label, required this.active, required this.onTap});

  final String label;
  final bool active;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) => GestureDetector(
        onTap: onTap,
        child: Container(
          padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 8),
          decoration: BoxDecoration(
            color: active ? VeyraColors.accent : VeyraColors.surface,
            borderRadius: BorderRadius.circular(999),
            border: Border.all(
              color: active ? VeyraColors.accent : VeyraColors.border,
            ),
          ),
          child: Text(
            label,
            style: TextStyle(
              color: active ? VeyraColors.bg : VeyraColors.textMuted,
              fontSize: 14,
              fontWeight: active ? FontWeight.w600 : FontWeight.w500,
            ),
          ),
        ),
      );
}
