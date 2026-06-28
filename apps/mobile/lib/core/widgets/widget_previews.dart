import 'package:flutter/material.dart';
import 'package:flutter/widget_previews.dart';
import 'package:veyra_mobile/core/theme/app_theme.dart';
import 'package:veyra_mobile/core/widgets/segmented_tabs.dart';
import 'package:veyra_mobile/core/widgets/skeleton.dart';
import 'package:veyra_mobile/core/widgets/status_pill.dart';

// Dark-first previews (the design register is dark) for the presentational leaf
// widgets. Open via the Flutter Widget Preview tool window with the apps/mobile
// module selected. Full screens are not previewed here — they need a
// ProviderScope + data and are exercised on a device instead.

/// Preview: the four [StatusPill] tones.
@Preview(name: 'StatusPill — tones', brightness: Brightness.dark)
Widget statusPillPreview() => const _Surface(
  child: Wrap(
    spacing: 8,
    runSpacing: 8,
    children: [
      StatusPill('Overdue', tone: PillTone.danger),
      StatusPill('Due soon', tone: PillTone.accent),
      StatusPill('Up to date', tone: PillTone.ok),
      StatusPill('None', tone: PillTone.muted),
    ],
  ),
);

/// Preview: the [SegmentedTabs] selector with the first tab active.
@Preview(name: 'SegmentedTabs', brightness: Brightness.dark)
Widget segmentedTabsPreview() => _Surface(
  child: SegmentedTabs(
    labels: const ['Reminders', 'Documents', 'Expenses'],
    index: 0,
    onChanged: (_) {},
  ),
);

/// Preview: stacked, animated [SkeletonBox] loaders.
@Preview(name: 'SkeletonBox', brightness: Brightness.dark)
Widget skeletonPreview() => const _Surface(
  child: Column(
    mainAxisSize: MainAxisSize.min,
    children: [
      SkeletonBox(height: 20, width: 180),
      SizedBox(height: 12),
      SkeletonBox(height: 20, width: 120),
      SizedBox(height: 12),
      SkeletonBox(height: 64),
    ],
  ),
);

/// Dark surface + padding shared by the previews above.
class _Surface extends StatelessWidget {
  const _Surface({required this.child});

  final Widget child;

  @override
  Widget build(BuildContext context) => ColoredBox(
    color: VeyraColors.bg,
    child: Padding(
      padding: const EdgeInsets.all(24),
      child: Center(child: child),
    ),
  );
}
