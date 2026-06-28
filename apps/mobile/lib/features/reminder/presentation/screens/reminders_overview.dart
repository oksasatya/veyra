import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:veyra_mobile/core/error/failure.dart';
import 'package:veyra_mobile/core/theme/app_theme.dart';
import 'package:veyra_mobile/core/widgets/skeleton.dart';
import 'package:veyra_mobile/core/widgets/status_pill.dart';
import 'package:veyra_mobile/features/reminder/data/repositories/reminder_repository_impl.dart';
import 'package:veyra_mobile/features/reminder/domain/entities/reminder.dart';
import 'package:veyra_mobile/features/reminder/domain/value_objects/reminder_type.dart';
import 'package:veyra_mobile/features/reminder/presentation/controllers/reminders_overview_controller.dart';

/// Cross-vehicle reminders, grouped by due status (design `reminders.html`).
class RemindersOverview extends ConsumerWidget {
  const RemindersOverview({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final reminders = ref.watch(remindersOverviewProvider);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Padding(
          padding: const EdgeInsets.fromLTRB(20, 16, 20, 8),
          child: Text('Reminders', style: soraDisplay(size: 30)),
        ),
        Expanded(
          child: reminders.when(
            loading: () => const _RemindersSkeleton(),
            error: (e, _) => _SectionError(
              message: e is Failure ? e.message : 'Could not load reminders.',
              onRetry: () => ref.invalidate(remindersOverviewProvider),
            ),
            data: (items) => items.isEmpty
                ? const _RemindersEmpty()
                : _GroupedList(items: items),
          ),
        ),
      ],
    );
  }
}

enum _Bucket { overdue, soon, upcoming, completed }

class _GroupedList extends StatelessWidget {
  const _GroupedList({required this.items});
  final List<ReminderWithVehicle> items;

  @override
  Widget build(BuildContext context) {
    final groups = <_Bucket, List<ReminderWithVehicle>>{
      for (final b in _Bucket.values) b: [],
    };
    for (final item in items) {
      groups[_bucketFor(item.reminder)]!.add(item);
    }
    const order = [
      (_Bucket.overdue, 'Overdue'),
      (_Bucket.soon, 'Due soon'),
      (_Bucket.upcoming, 'Upcoming'),
      (_Bucket.completed, 'Completed'),
    ];
    return ListView(
      padding: const EdgeInsets.fromLTRB(20, 6, 20, 96),
      children: [
        for (final (bucket, label) in order)
          if (groups[bucket]!.isNotEmpty) ...[
            _SectionLabel(label),
            for (final item in groups[bucket]!) _ReminderItem(item: item),
          ],
      ],
    );
  }
}

class _SectionLabel extends StatelessWidget {
  const _SectionLabel(this.text);
  final String text;

  @override
  Widget build(BuildContext context) => Padding(
        padding: const EdgeInsets.fromLTRB(2, 16, 2, 10),
        child: Text(
          text.toUpperCase(),
          style: const TextStyle(
            color: VeyraColors.textMuted,
            fontSize: 12,
            fontWeight: FontWeight.w600,
            letterSpacing: 0.6,
          ),
        ),
      );
}

class _ReminderItem extends ConsumerStatefulWidget {
  const _ReminderItem({required this.item});
  final ReminderWithVehicle item;

  @override
  ConsumerState<_ReminderItem> createState() => _ReminderItemState();
}

class _ReminderItemState extends ConsumerState<_ReminderItem> {
  bool _busy = false;

  Future<void> _complete() async {
    final r = widget.item.reminder;
    if (r.isCompleted || _busy) return;
    setState(() => _busy = true);
    final result =
        await ref.read(completeReminderUseCaseProvider)(r.vehicleId, r.id);
    if (!mounted) return;
    result.fold(
      (failure) {
        setState(() => _busy = false);
        ScaffoldMessenger.of(context)
            .showSnackBar(SnackBar(content: Text(failure.message)));
      },
      (_) => ref.invalidate(remindersOverviewProvider),
    );
  }

  @override
  Widget build(BuildContext context) {
    final r = widget.item.reminder;
    return Container(
      margin: const EdgeInsets.only(bottom: 10),
      padding: const EdgeInsets.symmetric(horizontal: 15, vertical: 13),
      decoration: BoxDecoration(
        color: VeyraColors.surface,
        borderRadius: BorderRadius.circular(14),
        border: Border.all(color: VeyraColors.border),
      ),
      child: Row(
        children: [
          _Check(done: r.isCompleted, busy: _busy, onTap: _complete),
          const SizedBox(width: 13),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  r.title,
                  style: TextStyle(
                    color: r.isCompleted
                        ? VeyraColors.textMuted
                        : VeyraColors.text,
                    fontSize: 15,
                    fontWeight: FontWeight.w500,
                    decoration:
                        r.isCompleted ? TextDecoration.lineThrough : null,
                  ),
                ),
                const SizedBox(height: 3),
                Text(_meta(widget.item), style: plexMono(size: 12)),
              ],
            ),
          ),
          if (!r.isCompleted) ...[
            const SizedBox(width: 10),
            _duePill(r),
          ],
        ],
      ),
    );
  }
}

class _Check extends StatelessWidget {
  const _Check({required this.done, required this.busy, required this.onTap});
  final bool done;
  final bool busy;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    if (busy) {
      return const SizedBox(
        width: 24,
        height: 24,
        child: CircularProgressIndicator(strokeWidth: 2.2),
      );
    }
    return Semantics(
      label: done ? 'Completed' : 'Mark complete',
      button: !done,
      child: GestureDetector(
        onTap: done ? null : onTap,
        child: Container(
          width: 24,
          height: 24,
          decoration: BoxDecoration(
            color: done ? VeyraColors.ok : Colors.transparent,
            shape: BoxShape.circle,
            border:
                Border.all(color: done ? VeyraColors.ok : VeyraColors.border, width: 2),
          ),
          child: done
              ? const Icon(Icons.check, size: 15, color: VeyraColors.bg)
              : null,
        ),
      ),
    );
  }
}

class _RemindersSkeleton extends StatelessWidget {
  const _RemindersSkeleton();

  @override
  Widget build(BuildContext context) => ListView(
        padding: const EdgeInsets.fromLTRB(20, 6, 20, 0),
        children: [
          for (var i = 0; i < 4; i++)
            const Padding(
              padding: EdgeInsets.only(bottom: 10),
              child: SkeletonBox(height: 64, radius: 14),
            ),
        ],
      );
}

class _RemindersEmpty extends StatelessWidget {
  const _RemindersEmpty();

  @override
  Widget build(BuildContext context) => const Center(
        child: Padding(
          padding: EdgeInsets.symmetric(horizontal: 40),
          child: Text(
            'No reminders yet. Add one from a vehicle to stay ahead of '
            'service and renewals.',
            textAlign: TextAlign.center,
            style: TextStyle(color: VeyraColors.textMuted, fontSize: 15),
          ),
        ),
      );
}

class _SectionError extends StatelessWidget {
  const _SectionError({required this.message, required this.onRetry});
  final String message;
  final VoidCallback onRetry;

  @override
  Widget build(BuildContext context) => Center(
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 40),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Text(
                message,
                textAlign: TextAlign.center,
                style: const TextStyle(
                  color: VeyraColors.textMuted,
                  fontSize: 15,
                ),
              ),
              const SizedBox(height: 16),
              FilledButton(onPressed: onRetry, child: const Text('Try again')),
            ],
          ),
        ),
      );
}

_Bucket _bucketFor(Reminder r) {
  if (r.isCompleted) return _Bucket.completed;
  final date = r.dueDate;
  if (date == null) return _Bucket.upcoming;
  final days = _daysUntil(date);
  if (days < 0) return _Bucket.overdue;
  if (days <= 14) return _Bucket.soon;
  return _Bucket.upcoming;
}

StatusPill _duePill(Reminder r) {
  final date = r.dueDate;
  if (date == null) {
    return StatusPill(_km(r.dueOdometer), tone: PillTone.muted);
  }
  final days = _daysUntil(date);
  if (days < 0) {
    final late = -days;
    return StatusPill(
      '$late day${late == 1 ? '' : 's'} late',
      tone: PillTone.danger,
    );
  }
  if (days <= 14) {
    return StatusPill('in $days day${days == 1 ? '' : 's'}',
        tone: PillTone.accent);
  }
  return StatusPill('in $days days', tone: PillTone.muted);
}

String _meta(ReminderWithVehicle item) {
  final r = item.reminder;
  final target = switch (r.type) {
    ReminderType.odometer => 'at ${_km(r.dueOdometer)}',
    _ => 'due ${_shortDate(r.dueDate)}',
  };
  return '${item.vehicleName} · $target';
}

int _daysUntil(DateTime date) {
  final now = DateTime.now();
  final today = DateTime(now.year, now.month, now.day);
  final target = DateTime(date.year, date.month, date.day);
  return target.difference(today).inDays;
}

const _months = [
  'Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun',
  'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec',
];

String _shortDate(DateTime? date) =>
    date == null ? '—' : '${date.day} ${_months[date.month - 1]}';

String _km(int? odometer) =>
    odometer == null ? '—' : '${_grouped(odometer)} km';

String _grouped(int n) {
  final s = n.toString();
  final buf = StringBuffer();
  for (var i = 0; i < s.length; i++) {
    if (i > 0 && (s.length - i) % 3 == 0) buf.write(',');
    buf.write(s[i]);
  }
  return buf.toString();
}
