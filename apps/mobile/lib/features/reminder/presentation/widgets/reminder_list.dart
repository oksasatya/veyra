import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:veyra_mobile/core/error/failure.dart';
import 'package:veyra_mobile/core/theme/app_theme.dart';
import 'package:veyra_mobile/features/reminder/data/repositories/reminder_repository_impl.dart';
import 'package:veyra_mobile/features/reminder/domain/entities/reminder.dart';
import 'package:veyra_mobile/features/reminder/domain/value_objects/reminder_type.dart';

const _ok = Color(0xFF4FD08A);

/// Lists a vehicle's reminders with a check control to mark each complete.
class ReminderList extends ConsumerWidget {
  const ReminderList({required this.vehicleId, super.key});
  final String vehicleId;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final reminders = ref.watch(reminderListProvider(vehicleId));
    return reminders.when(
      loading: () => const _RemindersSkeleton(),
      error: (e, _) => _RemindersError(
        message: e is Failure ? e.message : 'Could not load reminders.',
        onRetry: () => ref.invalidate(reminderListProvider(vehicleId)),
      ),
      data: (list) {
        if (list.isEmpty) return const _RemindersEmpty();
        return Column(
          children: [
            for (final reminder in list)
              _ReminderTile(vehicleId: vehicleId, reminder: reminder),
          ],
        );
      },
    );
  }
}

class _ReminderTile extends ConsumerStatefulWidget {
  const _ReminderTile({required this.vehicleId, required this.reminder});
  final String vehicleId;
  final Reminder reminder;

  @override
  ConsumerState<_ReminderTile> createState() => _ReminderTileState();
}

class _ReminderTileState extends ConsumerState<_ReminderTile> {
  bool _busy = false;

  Future<void> _complete() async {
    if (widget.reminder.isCompleted || _busy) return;
    setState(() => _busy = true);
    final result = await ref.read(completeReminderUseCaseProvider)(
      widget.vehicleId,
      widget.reminder.id,
    );
    if (!mounted) return;
    result.fold(
      (failure) {
        setState(() => _busy = false);
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text(failure.message)),
        );
      },
      // On success the list refreshes and this tile rebuilds as completed.
      (_) => ref.invalidate(reminderListProvider(widget.vehicleId)),
    );
  }

  @override
  Widget build(BuildContext context) {
    final r = widget.reminder;
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
          _CheckControl(
            done: r.isCompleted,
            busy: _busy,
            onTap: _complete,
          ),
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
                    decoration: r.isCompleted
                        ? TextDecoration.lineThrough
                        : null,
                  ),
                ),
                const SizedBox(height: 3),
                Text(_meta(r), style: plexMono(size: 12)),
              ],
            ),
          ),
          if (!r.isCompleted) ...[
            const SizedBox(width: 10),
            _DuePill(reminder: r),
          ],
        ],
      ),
    );
  }
}

class _CheckControl extends StatelessWidget {
  const _CheckControl({
    required this.done,
    required this.busy,
    required this.onTap,
  });
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
            color: done ? _ok : Colors.transparent,
            shape: BoxShape.circle,
            border: Border.all(
              color: done ? _ok : VeyraColors.border,
              width: 2,
            ),
          ),
          child: done
              ? const Icon(Icons.check, size: 15, color: VeyraColors.bg)
              : null,
        ),
      ),
    );
  }
}

class _DuePill extends StatelessWidget {
  const _DuePill({required this.reminder});
  final Reminder reminder;

  @override
  Widget build(BuildContext context) {
    final (label, color) = _due(reminder);
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 5),
      decoration: BoxDecoration(
        color: color.withValues(alpha: 0.14),
        borderRadius: BorderRadius.circular(999),
        border: Border.all(color: color.withValues(alpha: 0.32)),
      ),
      child: Text(
        label,
        style: TextStyle(
          color: color,
          fontSize: 12,
          fontWeight: FontWeight.w600,
        ),
      ),
    );
  }
}

class _RemindersSkeleton extends StatelessWidget {
  const _RemindersSkeleton();

  @override
  Widget build(BuildContext context) => Column(
    children: [
      for (var i = 0; i < 3; i++)
        Container(
          height: 64,
          margin: const EdgeInsets.only(bottom: 10),
          decoration: BoxDecoration(
            color: VeyraColors.surface,
            borderRadius: BorderRadius.circular(14),
            border: Border.all(color: VeyraColors.border),
          ),
        ),
    ],
  );
}

class _RemindersEmpty extends StatelessWidget {
  const _RemindersEmpty();

  @override
  Widget build(BuildContext context) => Container(
    padding: const EdgeInsets.symmetric(vertical: 28),
    alignment: Alignment.center,
    child: const Text(
      'No reminders yet. Add one to stay ahead of service & renewals.',
      textAlign: TextAlign.center,
      style: TextStyle(color: VeyraColors.textMuted, fontSize: 14),
    ),
  );
}

class _RemindersError extends StatelessWidget {
  const _RemindersError({required this.message, required this.onRetry});
  final String message;
  final VoidCallback onRetry;

  @override
  Widget build(BuildContext context) => Container(
    padding: const EdgeInsets.all(16),
    decoration: BoxDecoration(
      color: VeyraColors.surface,
      borderRadius: BorderRadius.circular(14),
      border: Border.all(color: VeyraColors.border),
    ),
    child: Row(
      children: [
        Expanded(
          child: Text(
            message,
            style: const TextStyle(
              color: VeyraColors.textMuted,
              fontSize: 13,
            ),
          ),
        ),
        TextButton(onPressed: onRetry, child: const Text('Retry')),
      ],
    ),
  );
}

const _months = [
  'Jan',
  'Feb',
  'Mar',
  'Apr',
  'May',
  'Jun',
  'Jul',
  'Aug',
  'Sep',
  'Oct',
  'Nov',
  'Dec',
];

/// Meta line: the reminder's type-appropriate due target.
String _meta(Reminder r) => switch (r.type) {
  ReminderType.date => 'By date · due ${_shortDate(r.dueDate)}',
  ReminderType.odometer => 'By odometer · at ${_km(r.dueOdometer)}',
  ReminderType.both =>
    'Date & odometer · ${_shortDate(r.dueDate)} / ${_km(r.dueOdometer)}',
};

/// Due pill label + colour, scaled by urgency for date-based reminders.
(String, Color) _due(Reminder r) {
  final date = r.dueDate;
  if (date == null) {
    return (_km(r.dueOdometer), VeyraColors.textMuted);
  }
  final days = _daysUntil(date);
  if (days < 0) {
    final late = -days;
    return ('$late day${late == 1 ? '' : 's'} late', VeyraColors.danger);
  }
  if (days <= 14) {
    return ('in $days day${days == 1 ? '' : 's'}', VeyraColors.accent);
  }
  return ('in $days days', VeyraColors.textMuted);
}

int _daysUntil(DateTime date) {
  final now = DateTime.now();
  final today = DateTime(now.year, now.month, now.day);
  final target = DateTime(date.year, date.month, date.day);
  return target.difference(today).inDays;
}

String _shortDate(DateTime? date) {
  if (date == null) return '—';
  return '${date.day} ${_months[date.month - 1]}';
}

String _km(int? odometer) {
  if (odometer == null) return '—';
  return '${_grouped(odometer)} km';
}

String _grouped(int n) {
  final s = n.toString();
  final buf = StringBuffer();
  for (var i = 0; i < s.length; i++) {
    if (i > 0 && (s.length - i) % 3 == 0) buf.write(',');
    buf.write(s[i]);
  }
  return buf.toString();
}
