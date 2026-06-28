import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:veyra_mobile/core/error/failure_l10n.dart';
import 'package:veyra_mobile/core/theme/app_theme.dart';
import 'package:veyra_mobile/features/vehicle/data/repositories/vehicle_repository_impl.dart';
import 'package:veyra_mobile/features/vehicle/domain/repositories/vehicle_repository.dart';
import 'package:veyra_mobile/features/vehicle/domain/value_objects/fuel_type.dart';
import 'package:veyra_mobile/features/vehicle/domain/value_objects/odometer.dart';
import 'package:veyra_mobile/features/vehicle/domain/value_objects/plate_number.dart';
import 'package:veyra_mobile/features/vehicle/presentation/controllers/garage_dashboard_controller.dart';
import 'package:veyra_mobile/features/vehicle/presentation/vehicle_l10n.dart';
import 'package:veyra_mobile/l10n/app_localizations.dart';

class AddVehicleScreen extends ConsumerStatefulWidget {
  const AddVehicleScreen({super.key});

  @override
  ConsumerState<AddVehicleScreen> createState() => _AddVehicleScreenState();
}

class _AddVehicleScreenState extends ConsumerState<AddVehicleScreen> {
  final _brand = TextEditingController();
  final _model = TextEditingController();
  final _year = TextEditingController();
  final _plate = TextEditingController();
  final _odometer = TextEditingController();
  final _color = TextEditingController();
  FuelType _fuel = FuelType.petrol;
  String? _error;
  bool _saving = false;

  @override
  void dispose() {
    _brand.dispose();
    _model.dispose();
    _year.dispose();
    _plate.dispose();
    _odometer.dispose();
    _color.dispose();
    super.dispose();
  }

  Future<void> _submit() async {
    final l10n = AppLocalizations.of(context);
    if (_brand.text.trim().isEmpty || _model.text.trim().isEmpty) {
      setState(() => _error = l10n.vehicleAddErrorBrandModel);
      return;
    }
    final year = int.tryParse(_year.text.trim());
    if (year == null || year < 1900 || year > 2100) {
      setState(() => _error = l10n.vehicleAddErrorYear);
      return;
    }
    final plate = PlateNumber.create(_plate.text).toNullable();
    if (plate == null) {
      setState(() => _error = l10n.vehicleAddErrorPlate);
      return;
    }
    final odo = Odometer.create(
      int.tryParse(_odometer.text.trim()) ?? -1,
    ).toNullable();
    if (odo == null) {
      setState(() => _error = l10n.vehicleAddErrorOdometer);
      return;
    }

    setState(() {
      _error = null;
      _saving = true;
    });
    final color = _color.text.trim();
    final result = await ref.read(createVehicleUseCaseProvider)(
      CreateVehicleInput(
        brand: _brand.text.trim(),
        model: _model.text.trim(),
        year: year,
        plateNumber: plate.value,
        fuelType: _fuel,
        odometer: odo.value,
        color: color.isEmpty ? null : color,
      ),
    );
    if (!mounted) return;
    final postL10n = AppLocalizations.of(context);
    result.fold(
      (failure) => setState(() {
        _error = localizedFailure(postL10n, failure);
        _saving = false;
      }),
      (_) {
        ref.invalidate(garageDashboardProvider);
        context.pop();
      },
    );
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    return Scaffold(
      appBar: AppBar(title: Text(l10n.vehicleAddTitle, style: soraDisplay(size: 18))),
      body: SafeArea(
        child: ListView(
          padding: const EdgeInsets.fromLTRB(20, 8, 20, 32),
          children: [
            _field(l10n.vehicleAddFieldBrand, _brand, hint: 'Toyota'),
            _field(l10n.vehicleAddFieldModel, _model, hint: 'Avanza'),
            _field(l10n.vehicleAddFieldYear, _year, hint: '2020', number: true),
            _field(l10n.vehicleAddFieldPlate, _plate, hint: 'B 1234 XYZ'),
            _Label(l10n.vehicleAddFieldFuelType),
            _FuelSelector(
              value: _fuel,
              l10n: l10n,
              onChanged: (f) => setState(() => _fuel = f),
            ),
            const SizedBox(height: 14),
            _field(l10n.vehicleAddFieldOdometer, _odometer, hint: '0', number: true),
            _field(l10n.vehicleAddFieldColor, _color, hint: 'Silver'),
            if (_error != null) ...[
              const SizedBox(height: 6),
              Text(
                _error!,
                style: const TextStyle(color: VeyraColors.danger, fontSize: 13),
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
                  : Text(l10n.vehicleAddSave),
            ),
          ],
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

class _FuelSelector extends StatelessWidget {
  const _FuelSelector({
    required this.value,
    required this.l10n,
    required this.onChanged,
  });
  final FuelType value;
  final AppLocalizations l10n;
  final ValueChanged<FuelType> onChanged;

  @override
  Widget build(BuildContext context) => Wrap(
    spacing: 8,
    children: [
      for (final f in FuelType.values)
        ChoiceChip(
          label: Text(localizedFuelType(l10n, f)),
          selected: f == value,
          onSelected: (_) => onChanged(f),
          backgroundColor: VeyraColors.surface,
          selectedColor: VeyraColors.accent,
          labelStyle: TextStyle(
            color: f == value ? VeyraColors.bg : VeyraColors.text,
            fontWeight: FontWeight.w500,
          ),
          side: const BorderSide(color: VeyraColors.border),
        ),
    ],
  );
}
