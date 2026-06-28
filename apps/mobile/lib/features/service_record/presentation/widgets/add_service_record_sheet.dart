import 'package:decimal/decimal.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:veyra_mobile/core/error/failure_l10n.dart';
import 'package:veyra_mobile/core/theme/app_theme.dart';
import 'package:veyra_mobile/features/service_record/data/repositories/service_record_repository_impl.dart';
import 'package:veyra_mobile/features/service_record/domain/repositories/service_record_repository.dart';
import 'package:veyra_mobile/l10n/app_localizations.dart';

/// Bottom-sheet form to log a service record: date, odometer, description,
/// optional workshop, optional cost, optional notes.
class AddServiceRecordSheet extends ConsumerStatefulWidget {
  const AddServiceRecordSheet({required this.vehicleId, super.key});
  final String vehicleId;

  @override
  ConsumerState<AddServiceRecordSheet> createState() =>
      _AddServiceRecordSheetState();
}

class _AddServiceRecordSheetState extends ConsumerState<AddServiceRecordSheet> {
  final _odometer = TextEditingController();
  final _description = TextEditingController();
  final _workshop = TextEditingController();
  final _cost = TextEditingController();
  final _notes = TextEditingController();
  DateTime _date = DateTime.now();
  String? _error;
  bool _saving = false;

  @override
  void dispose() {
    _odometer.dispose();
    _description.dispose();
    _workshop.dispose();
    _cost.dispose();
    _notes.dispose();
    super.dispose();
  }

  Future<void> _pickDate() async {
    final picked = await showDatePicker(
      context: context,
      initialDate: _date,
      firstDate: DateTime(1990),
      lastDate: DateTime(2100),
    );
    if (picked != null) setState(() => _date = picked);
  }

  Future<void> _submit() async {
    final l10n = AppLocalizations.of(context);
    if (_description.text.trim().isEmpty) {
      setState(() => _error = l10n.serviceRecordErrorDescription);
      return;
    }
    final odo = int.tryParse(_odometer.text.trim());
    if (odo == null || odo < 0) {
      setState(() => _error = l10n.serviceRecordErrorOdometer);
      return;
    }
    Decimal? cost;
    final costText = _cost.text.trim();
    if (costText.isNotEmpty) {
      cost = Decimal.tryParse(costText);
      if (cost == null || cost < Decimal.zero) {
        setState(() => _error = l10n.serviceRecordErrorCost);
        return;
      }
    }

    setState(() {
      _error = null;
      _saving = true;
    });
    final workshop = _workshop.text.trim();
    final notes = _notes.text.trim();
    final result = await ref.read(createServiceRecordUseCaseProvider)(
      CreateServiceRecordInput(
        vehicleId: widget.vehicleId,
        serviceDate: _date,
        odometer: odo,
        description: _description.text.trim(),
        workshop: workshop.isEmpty ? null : workshop,
        cost: cost,
        notes: notes.isEmpty ? null : notes,
      ),
    );
    if (!mounted) return;
    final l10nAfter = AppLocalizations.of(context);
    result.fold(
      (failure) => setState(() {
        _error = localizedFailure(l10nAfter, failure);
        _saving = false;
      }),
      (_) {
        ref.invalidate(serviceRecordListProvider(widget.vehicleId));
        Navigator.of(context).pop();
      },
    );
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    return Padding(
      padding: EdgeInsets.only(
        bottom: MediaQuery.of(context).viewInsets.bottom,
      ),
      child: SafeArea(
        child: SingleChildScrollView(
          padding: const EdgeInsets.fromLTRB(20, 16, 20, 24),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(l10n.serviceRecordTitle, style: soraDisplay(size: 20)),
              const SizedBox(height: 18),
              _Label(l10n.serviceRecordFieldDate),
              _DateField(date: _date, onTap: _pickDate),
              const SizedBox(height: 14),
              _field(l10n.serviceRecordFieldOdometer, _odometer,
                  hint: '0', number: true),
              _field(l10n.serviceRecordFieldDescription, _description,
                  hint: 'Oil change'),
              _field(l10n.serviceRecordFieldWorkshop, _workshop,
                  hint: 'AutoCare'),
              _field(l10n.serviceRecordFieldCost, _cost,
                  hint: '350000', number: true),
              _field(l10n.serviceRecordFieldNotes, _notes,
                  hint: 'Synthetic oil'),
              if (_error != null) ...[
                const SizedBox(height: 6),
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
                    : Text(l10n.serviceRecordSave),
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _field(
    String label,
    TextEditingController controller, {
    String? hint,
    bool number = false,
  }) => Column(
    crossAxisAlignment: CrossAxisAlignment.start,
    children: [
      _Label(label),
      TextField(
        controller: controller,
        keyboardType: number ? TextInputType.number : TextInputType.text,
        decoration: InputDecoration(hintText: hint),
      ),
      const SizedBox(height: 14),
    ],
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

class _DateField extends StatelessWidget {
  const _DateField({required this.date, required this.onTap});
  final DateTime date;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) => InkWell(
    onTap: onTap,
    borderRadius: BorderRadius.circular(14),
    child: Container(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 16),
      decoration: BoxDecoration(
        color: VeyraColors.surface,
        borderRadius: BorderRadius.circular(14),
        border: Border.all(color: VeyraColors.border),
      ),
      child: Row(
        children: [
          Text(
            _format(date),
            style: plexMono(size: 14, color: VeyraColors.text),
          ),
          const Spacer(),
          const Icon(
            Icons.calendar_today,
            size: 16,
            color: VeyraColors.textMuted,
          ),
        ],
      ),
    ),
  );

  static String _format(DateTime d) =>
      '${d.year.toString().padLeft(4, '0')}-'
      '${d.month.toString().padLeft(2, '0')}-'
      '${d.day.toString().padLeft(2, '0')}';
}
