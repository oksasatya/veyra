import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:veyra_mobile/core/theme/app_theme.dart';
import 'package:veyra_mobile/features/reminder/data/repositories/reminder_repository_impl.dart';
import 'package:veyra_mobile/features/reminder/domain/usecases/validate_reminder.dart';
import 'package:veyra_mobile/features/reminder/domain/value_objects/reminder_type.dart';

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

/// Bottom-sheet form to add a reminder for [vehicleId]. The due fields shown
/// adapt to the selected [ReminderType] (date / odometer / both).
class AddReminderSheet extends ConsumerStatefulWidget {
  const AddReminderSheet({required this.vehicleId, super.key});
  final String vehicleId;

  @override
  ConsumerState<AddReminderSheet> createState() => _AddReminderSheetState();
}

class _AddReminderSheetState extends ConsumerState<AddReminderSheet> {
  final _title = TextEditingController();
  final _odometer = TextEditingController();
  final _notes = TextEditingController();
  ReminderType _type = ReminderType.date;
  DateTime? _dueDate;
  String? _error;
  bool _saving = false;

  bool get _needsDate =>
      _type == ReminderType.date || _type == ReminderType.both;
  bool get _needsOdometer =>
      _type == ReminderType.odometer || _type == ReminderType.both;

  @override
  void dispose() {
    _title.dispose();
    _odometer.dispose();
    _notes.dispose();
    super.dispose();
  }

  Future<void> _pickDate() async {
    final now = DateTime.now();
    final picked = await showDatePicker(
      context: context,
      initialDate: _dueDate ?? now,
      firstDate: DateTime(now.year - 1),
      lastDate: DateTime(now.year + 10),
    );
    if (picked != null) setState(() => _dueDate = picked);
  }

  Future<void> _submit() async {
    final odometer = _needsOdometer
        ? int.tryParse(_odometer.text.trim())
        : null;
    final validated = validateReminder(
      vehicleId: widget.vehicleId,
      title: _title.text,
      type: _type,
      dueDate: _needsDate ? _dueDate : null,
      dueOdometer: odometer,
      notes: _notes.text,
    );

    final input = validated.toNullable();
    if (input == null) {
      setState(
        () => _error = validated.getLeft().toNullable()?.message,
      );
      return;
    }

    setState(() {
      _error = null;
      _saving = true;
    });
    final result = await ref.read(createReminderUseCaseProvider)(input);
    if (!mounted) return;
    result.fold(
      (failure) => setState(() {
        _error = failure.message;
        _saving = false;
      }),
      (_) {
        ref.invalidate(reminderListProvider(widget.vehicleId));
        Navigator.of(context).pop();
      },
    );
  }

  @override
  Widget build(BuildContext context) {
    final inset = MediaQuery.of(context).viewInsets.bottom;
    return Padding(
      padding: EdgeInsets.only(bottom: inset),
      child: Container(
        decoration: const BoxDecoration(
          color: VeyraColors.bg,
          borderRadius: BorderRadius.vertical(top: Radius.circular(20)),
        ),
        child: SafeArea(
          top: false,
          child: SingleChildScrollView(
            padding: const EdgeInsets.fromLTRB(20, 14, 20, 20),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                const _Grabber(),
                const SizedBox(height: 14),
                Text('Add reminder', style: soraDisplay(size: 20)),
                const SizedBox(height: 18),
                const _Label('Title'),
                TextField(
                  controller: _title,
                  decoration: const InputDecoration(
                    hintText: 'Road tax renewal',
                  ),
                ),
                const SizedBox(height: 16),
                const _Label('Trigger'),
                _TypeSelector(
                  value: _type,
                  onChanged: (t) => setState(() => _type = t),
                ),
                const SizedBox(height: 16),
                if (_needsDate) ...[
                  const _Label('Due date'),
                  _DateField(value: _dueDate, onTap: _pickDate),
                  const SizedBox(height: 16),
                ],
                if (_needsOdometer) ...[
                  const _Label('Due odometer (km)'),
                  TextField(
                    controller: _odometer,
                    keyboardType: TextInputType.number,
                    decoration: const InputDecoration(hintText: '129000'),
                  ),
                  const SizedBox(height: 16),
                ],
                const _Label('Notes (optional)'),
                TextField(
                  controller: _notes,
                  maxLines: 2,
                  decoration: const InputDecoration(
                    hintText: 'Anything to note',
                  ),
                ),
                if (_error != null) ...[
                  const SizedBox(height: 10),
                  Text(
                    _error!,
                    style: const TextStyle(
                      color: VeyraColors.danger,
                      fontSize: 13,
                    ),
                  ),
                ],
                const SizedBox(height: 22),
                FilledButton(
                  onPressed: _saving ? null : _submit,
                  child: _saving
                      ? const SizedBox(
                          height: 22,
                          width: 22,
                          child: CircularProgressIndicator(
                            strokeWidth: 2.4,
                            color: VeyraColors.bg,
                          ),
                        )
                      : const Text('Save reminder'),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

class _Grabber extends StatelessWidget {
  const _Grabber();

  @override
  Widget build(BuildContext context) => Center(
    child: Container(
      width: 40,
      height: 4,
      decoration: BoxDecoration(
        color: VeyraColors.border,
        borderRadius: BorderRadius.circular(2),
      ),
    ),
  );
}

class _Label extends StatelessWidget {
  const _Label(this.text);
  final String text;

  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.only(bottom: 8),
    child: Text(
      text,
      style: const TextStyle(
        color: VeyraColors.textMuted,
        fontSize: 13,
        fontWeight: FontWeight.w500,
      ),
    ),
  );
}

class _TypeSelector extends StatelessWidget {
  const _TypeSelector({required this.value, required this.onChanged});
  final ReminderType value;
  final ValueChanged<ReminderType> onChanged;

  @override
  Widget build(BuildContext context) => Wrap(
    spacing: 8,
    children: [
      for (final t in ReminderType.values)
        ChoiceChip(
          label: Text(t.label),
          selected: t == value,
          onSelected: (_) => onChanged(t),
          backgroundColor: VeyraColors.surface,
          selectedColor: VeyraColors.accent,
          labelStyle: TextStyle(
            color: t == value ? VeyraColors.bg : VeyraColors.text,
            fontWeight: FontWeight.w500,
          ),
          side: const BorderSide(color: VeyraColors.border),
        ),
    ],
  );
}

class _DateField extends StatelessWidget {
  const _DateField({required this.value, required this.onTap});
  final DateTime? value;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) => InkWell(
    onTap: onTap,
    borderRadius: BorderRadius.circular(14),
    child: Container(
      width: double.infinity,
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 16),
      decoration: BoxDecoration(
        color: VeyraColors.surface,
        borderRadius: BorderRadius.circular(14),
        border: Border.all(color: VeyraColors.border),
      ),
      child: Row(
        children: [
          Text(
            value == null ? 'Pick a date' : _format(value!),
            style: TextStyle(
              color: value == null ? const Color(0xFF5A6472) : VeyraColors.text,
              fontSize: 16,
            ),
          ),
          const Spacer(),
          const Icon(
            Icons.calendar_today_outlined,
            size: 18,
            color: VeyraColors.textMuted,
          ),
        ],
      ),
    ),
  );

  String _format(DateTime d) => '${d.day} ${_months[d.month - 1]} ${d.year}';
}
