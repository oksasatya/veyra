import 'package:decimal/decimal.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:veyra_mobile/core/error/failure_l10n.dart';
import 'package:veyra_mobile/core/theme/app_theme.dart';
import 'package:veyra_mobile/features/fuel_log/data/repositories/fuel_log_repository_impl.dart';
import 'package:veyra_mobile/features/fuel_log/domain/repositories/fuel_log_repository.dart';
import 'package:veyra_mobile/features/fuel_log/domain/value_objects/positive_decimal.dart';
import 'package:veyra_mobile/features/vehicle/domain/value_objects/odometer.dart';
import 'package:veyra_mobile/l10n/app_localizations.dart';

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

/// Bottom-sheet form to log a fuel fill-up for [vehicleId]. Computes the total
/// (liters × price) live, validates via value objects, creates the log, then
/// invalidates the list provider and pops on success.
class AddFuelLogSheet extends ConsumerStatefulWidget {
  const AddFuelLogSheet({required this.vehicleId, super.key});
  final String vehicleId;

  @override
  ConsumerState<AddFuelLogSheet> createState() => _AddFuelLogSheetState();
}

class _AddFuelLogSheetState extends ConsumerState<AddFuelLogSheet> {
  final _odometer = TextEditingController();
  final _liters = TextEditingController();
  final _price = TextEditingController();
  final _station = TextEditingController();
  DateTime _date = DateTime.now();
  bool _isFullTank = true;
  String? _error;
  bool _saving = false;

  @override
  void initState() {
    super.initState();
    _liters.addListener(_onChanged);
    _price.addListener(_onChanged);
  }

  @override
  void dispose() {
    _odometer.dispose();
    _liters.dispose();
    _price.dispose();
    _station.dispose();
    super.dispose();
  }

  void _onChanged() => setState(() {});

  Decimal? get _total {
    final liters = Decimal.tryParse(_liters.text.trim());
    final price = Decimal.tryParse(_price.text.trim());
    if (liters == null || price == null) return null;
    return liters * price;
  }

  Future<void> _pickDate() async {
    final picked = await showDatePicker(
      context: context,
      initialDate: _date,
      firstDate: DateTime(2000),
      lastDate: DateTime.now(),
    );
    if (picked != null) setState(() => _date = picked);
  }

  Future<void> _submit() async {
    final l10n = AppLocalizations.of(context);
    final odo = Odometer.create(
      int.tryParse(_odometer.text.trim()) ?? -1,
    ).toNullable();
    if (odo == null) {
      setState(() => _error = l10n.fuelLogErrorOdometer);
      return;
    }
    final liters = PositiveDecimal.create(_liters.text, field: 'liters');
    final litersValue = liters.toNullable();
    if (litersValue == null) {
      setState(() => _error = l10n.fuelLogErrorLiters);
      return;
    }
    final price = PositiveDecimal.create(_price.text, field: 'price');
    final priceValue = price.toNullable();
    if (priceValue == null) {
      setState(() => _error = l10n.fuelLogErrorPricePerLiter);
      return;
    }

    setState(() {
      _error = null;
      _saving = true;
    });
    final station = _station.text.trim();
    final result = await ref.read(createFuelLogUseCaseProvider)(
      CreateFuelLogInput(
        vehicleId: widget.vehicleId,
        logDate: _date,
        odometer: odo.value,
        liters: litersValue,
        pricePerLiter: priceValue,
        station: station.isEmpty ? null : station,
        isFullTank: _isFullTank,
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
        ref.invalidate(fuelLogListProvider(widget.vehicleId));
        Navigator.of(context).pop();
      },
    );
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final total = _total;
    final bottomInset = MediaQuery.of(context).viewInsets.bottom;
    return Padding(
      padding: EdgeInsets.only(bottom: bottomInset),
      child: Container(
        decoration: const BoxDecoration(
          color: VeyraColors.surface,
          borderRadius: BorderRadius.vertical(top: Radius.circular(26)),
          border: Border(top: BorderSide(color: VeyraColors.border)),
        ),
        child: SafeArea(
          top: false,
          child: SingleChildScrollView(
            padding: const EdgeInsets.fromLTRB(22, 12, 22, 26),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                const _Grabber(),
                const SizedBox(height: 10),
                Row(
                  children: [
                    Expanded(
                      child: Text(
                        l10n.fuelLogTitle,
                        style: soraDisplay(size: 21),
                      ),
                    ),
                    IconButton(
                      onPressed: () => Navigator.of(context).pop(),
                      icon: const Icon(
                        Icons.close,
                        color: VeyraColors.textMuted,
                      ),
                    ),
                  ],
                ),
                const SizedBox(height: 8),
                Row(
                  children: [
                    Expanded(
                      child: _DateField(
                        label: l10n.fuelLogFieldDate,
                        date: _date,
                        onTap: _pickDate,
                      ),
                    ),
                    const SizedBox(width: 12),
                    Expanded(
                      child: _LabeledField(
                        label: l10n.fuelLogFieldOdometer,
                        controller: _odometer,
                        hint: '0',
                        suffix: 'km',
                        number: true,
                      ),
                    ),
                  ],
                ),
                const SizedBox(height: 14),
                Row(
                  children: [
                    Expanded(
                      child: _LabeledField(
                        label: l10n.fuelLogFieldLiters,
                        controller: _liters,
                        hint: '0.0',
                        suffix: 'L',
                        number: true,
                      ),
                    ),
                    const SizedBox(width: 12),
                    Expanded(
                      child: _LabeledField(
                        label: l10n.fuelLogFieldPricePerLiter,
                        controller: _price,
                        hint: '0',
                        suffix: 'Rp',
                        number: true,
                      ),
                    ),
                  ],
                ),
                const SizedBox(height: 18),
                _TotalRow(label: l10n.fuelLogFieldTotalCost, total: total),
                const SizedBox(height: 14),
                _LabeledField(
                  label: l10n.fuelLogFieldStation,
                  controller: _station,
                  hint: 'Pertamina',
                ),
                const SizedBox(height: 18),
                _FullTankToggle(
                  label: l10n.fuelLogFieldFullTank,
                  hint: l10n.fuelLogFieldFullTankHint,
                  value: _isFullTank,
                  onChanged: (v) => setState(() => _isFullTank = v),
                ),
                if (_error != null) ...[
                  const SizedBox(height: 12),
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
                      : Text(l10n.fuelLogSave),
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
      height: 5,
      decoration: BoxDecoration(
        color: const Color(0xFF39424F),
        borderRadius: BorderRadius.circular(3),
      ),
    ),
  );
}

class _Label extends StatelessWidget {
  const _Label(this.text);
  final String text;

  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.only(bottom: 7),
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

class _LabeledField extends StatelessWidget {
  const _LabeledField({
    required this.label,
    required this.controller,
    this.hint,
    this.suffix,
    this.number = false,
  });

  final String label;
  final TextEditingController controller;
  final String? hint;
  final String? suffix;
  final bool number;

  @override
  Widget build(BuildContext context) => Column(
    crossAxisAlignment: CrossAxisAlignment.start,
    children: [
      _Label(label),
      TextField(
        controller: controller,
        keyboardType: number
            ? const TextInputType.numberWithOptions(decimal: true)
            : TextInputType.text,
        style: number ? plexMono(size: 16, color: VeyraColors.text) : null,
        decoration: InputDecoration(
          hintText: hint,
          suffixText: suffix,
          suffixStyle: const TextStyle(
            color: VeyraColors.textMuted,
            fontSize: 13,
          ),
        ),
      ),
    ],
  );
}

class _DateField extends StatelessWidget {
  const _DateField({
    required this.label,
    required this.date,
    required this.onTap,
  });
  final String label;
  final DateTime date;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) => Column(
    crossAxisAlignment: CrossAxisAlignment.start,
    children: [
      _Label(label),
      InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(14),
        child: Container(
          height: 54,
          padding: const EdgeInsets.symmetric(horizontal: 16),
          alignment: Alignment.centerLeft,
          decoration: BoxDecoration(
            color: VeyraColors.surface,
            borderRadius: BorderRadius.circular(14),
            border: Border.all(color: VeyraColors.border),
          ),
          child: Text(
            '${date.day} ${_months[date.month - 1]} ${date.year}',
            style: plexMono(size: 16, color: VeyraColors.text),
          ),
        ),
      ),
    ],
  );
}

class _TotalRow extends StatelessWidget {
  const _TotalRow({required this.label, required this.total});
  final String label;
  final Decimal? total;

  @override
  Widget build(BuildContext context) => Container(
    padding: const EdgeInsets.symmetric(horizontal: 17, vertical: 15),
    decoration: BoxDecoration(
      color: VeyraColors.surface2,
      borderRadius: BorderRadius.circular(14),
      border: Border.all(color: VeyraColors.border),
    ),
    child: Row(
      children: [
        Expanded(
          child: Text(
            label,
            style: const TextStyle(
              color: VeyraColors.textMuted,
              fontSize: 13,
            ),
          ),
        ),
        Text(
          total == null ? 'Rp —' : _money(total!),
          style: plexMono(size: 22, color: VeyraColors.accent),
        ),
      ],
    ),
  );
}

class _FullTankToggle extends StatelessWidget {
  const _FullTankToggle({
    required this.label,
    required this.hint,
    required this.value,
    required this.onChanged,
  });
  final String label;
  final String hint;
  final bool value;
  final ValueChanged<bool> onChanged;

  @override
  Widget build(BuildContext context) => Row(
    children: [
      Expanded(
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              label,
              style: const TextStyle(
                color: VeyraColors.text,
                fontSize: 15,
                fontWeight: FontWeight.w500,
              ),
            ),
            const SizedBox(height: 3),
            Text(
              hint,
              style: const TextStyle(
                color: VeyraColors.textMuted,
                fontSize: 12,
              ),
            ),
          ],
        ),
      ),
      Switch(
        value: value,
        onChanged: onChanged,
        activeThumbColor: VeyraColors.bg,
        activeTrackColor: VeyraColors.accent,
      ),
    ],
  );
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

String _money(Decimal d) => 'Rp ${_grouped(d.round().toBigInt().toInt())}';
