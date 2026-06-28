import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:veyra_mobile/core/error/failure.dart';
import 'package:veyra_mobile/core/theme/app_theme.dart';
import 'package:veyra_mobile/core/widgets/veyra_mark.dart';
import 'package:veyra_mobile/features/auth/presentation/controllers/auth_controller.dart';
import 'package:veyra_mobile/features/vehicle/domain/entities/vehicle.dart';
import 'package:veyra_mobile/features/vehicle/presentation/controllers/garage_controller.dart';

class GarageScreen extends ConsumerWidget {
  const GarageScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final vehicles = ref.watch(garageControllerProvider);
    return Scaffold(
      floatingActionButton: FloatingActionButton.extended(
        onPressed: () => context.push('/vehicles/new'),
        backgroundColor: VeyraColors.accent,
        foregroundColor: VeyraColors.bg,
        icon: const Icon(Icons.add),
        label: const Text('Add vehicle'),
      ),
      body: SafeArea(
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Padding(
              padding: const EdgeInsets.fromLTRB(20, 12, 12, 4),
              child: Row(
                children: [
                  Expanded(child: Text('Garage', style: soraDisplay(size: 32))),
                  IconButton(
                    onPressed: () =>
                        ref.read(authControllerProvider.notifier).logout(),
                    icon: const Icon(Icons.logout, color: VeyraColors.textMuted),
                  ),
                ],
              ),
            ),
            Expanded(
              child: vehicles.when(
                loading: () => const _GarageSkeleton(),
                error: (e, _) => _ErrorState(
                  message: e is Failure ? e.message : 'Something went wrong.',
                  onRetry: () => ref.read(garageControllerProvider.notifier).refresh(),
                ),
                data: (list) => list.isEmpty
                    ? const _EmptyGarage()
                    : _VehicleList(
                        vehicles: list,
                        onRefresh: () =>
                            ref.read(garageControllerProvider.notifier).refresh(),
                        onTap: (v) => context.push('/vehicles/${v.id}', extra: v),
                      ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class _VehicleList extends StatelessWidget {
  const _VehicleList({
    required this.vehicles,
    required this.onRefresh,
    required this.onTap,
  });
  final List<Vehicle> vehicles;
  final Future<void> Function() onRefresh;
  final void Function(Vehicle) onTap;

  @override
  Widget build(BuildContext context) => RefreshIndicator(
        onRefresh: onRefresh,
        color: VeyraColors.accent,
        backgroundColor: VeyraColors.surface,
        child: ListView(
          padding: const EdgeInsets.fromLTRB(20, 8, 20, 96),
          children: [
            _Overview(count: vehicles.length),
            const SizedBox(height: 18),
            for (final v in vehicles) ...[
              _VehicleCard(vehicle: v, onTap: () => onTap(v)),
              const SizedBox(height: 14),
            ],
          ],
        ),
      );
}

class _Overview extends StatelessWidget {
  const _Overview({required this.count});
  final int count;

  @override
  Widget build(BuildContext context) => Container(
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 14),
        decoration: BoxDecoration(
          color: VeyraColors.surface,
          borderRadius: BorderRadius.circular(14),
          border: Border.all(color: VeyraColors.border),
        ),
        child: Row(
          children: [
            const Text(
              'Vehicles',
              style: TextStyle(color: VeyraColors.textMuted, fontSize: 13),
            ),
            const Spacer(),
            Text('$count', style: soraDisplay(size: 18)),
          ],
        ),
      );
}

class _VehicleCard extends StatelessWidget {
  const _VehicleCard({required this.vehicle, required this.onTap});
  final Vehicle vehicle;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) => Material(
        color: Colors.transparent,
        child: InkWell(
          onTap: onTap,
          borderRadius: BorderRadius.circular(18),
          child: Ink(
            padding: const EdgeInsets.all(16),
            decoration: BoxDecoration(
              color: VeyraColors.surface,
              borderRadius: BorderRadius.circular(18),
              border: Border.all(color: VeyraColors.border),
            ),
            child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Container(
                  width: 44,
                  height: 44,
                  decoration: BoxDecoration(
                    color: VeyraColors.surface2,
                    borderRadius: BorderRadius.circular(12),
                    border: Border.all(color: VeyraColors.border),
                  ),
                  child: const Icon(
                    Icons.directions_car_outlined,
                    color: VeyraColors.accent,
                    size: 22,
                  ),
                ),
                const SizedBox(width: 12),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(vehicle.displayName, style: soraDisplay(size: 17)),
                      const SizedBox(height: 3),
                      Text(
                        '${vehicle.plateNumber} · ${vehicle.year} · ${vehicle.fuelType.label}',
                        style: plexMono(size: 13),
                      ),
                    ],
                  ),
                ),
              ],
            ),
            const SizedBox(height: 13),
            Text(
              'Odometer ${_formatKm(vehicle.odometer)} km',
              style: const TextStyle(color: VeyraColors.textMuted, fontSize: 13),
            ),
              ],
            ),
          ),
        ),
      );

  String _formatKm(int km) {
    final s = km.toString();
    final buf = StringBuffer();
    for (var i = 0; i < s.length; i++) {
      if (i > 0 && (s.length - i) % 3 == 0) buf.write(',');
      buf.write(s[i]);
    }
    return buf.toString();
  }
}

class _EmptyGarage extends StatelessWidget {
  const _EmptyGarage();

  @override
  Widget build(BuildContext context) => Center(
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 40),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Container(
                width: 108,
                height: 108,
                decoration: BoxDecoration(
                  color: VeyraColors.surface,
                  borderRadius: BorderRadius.circular(28),
                  border: Border.all(color: VeyraColors.border),
                ),
                alignment: Alignment.center,
                child: const VeyraMark(size: 52),
              ),
              const SizedBox(height: 24),
              Text('Add your first vehicle', style: soraDisplay(size: 21)),
              const SizedBox(height: 10),
              const Text(
                'Track services, fuel, expenses, and reminders once your car '
                'or bike is in the garage.',
                textAlign: TextAlign.center,
                style: TextStyle(
                  color: VeyraColors.textMuted,
                  fontSize: 15,
                  height: 1.5,
                ),
              ),
            ],
          ),
        ),
      );
}

class _ErrorState extends StatelessWidget {
  const _ErrorState({required this.message, required this.onRetry});
  final String message;
  final VoidCallback onRetry;

  @override
  Widget build(BuildContext context) => Center(
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 40),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              const Icon(Icons.cloud_off, color: VeyraColors.danger, size: 40),
              const SizedBox(height: 16),
              Text("Can't load your garage", style: soraDisplay(size: 20)),
              const SizedBox(height: 10),
              Text(
                message,
                textAlign: TextAlign.center,
                style: const TextStyle(
                  color: VeyraColors.textMuted,
                  fontSize: 15,
                  height: 1.5,
                ),
              ),
              const SizedBox(height: 22),
              FilledButton(onPressed: onRetry, child: const Text('Try again')),
            ],
          ),
        ),
      );
}

class _GarageSkeleton extends StatelessWidget {
  const _GarageSkeleton();

  @override
  Widget build(BuildContext context) => ListView(
        padding: const EdgeInsets.fromLTRB(20, 8, 20, 20),
        children: [
          for (var i = 0; i < 3; i++) ...[
            Container(
              height: 116,
              decoration: BoxDecoration(
                color: VeyraColors.surface,
                borderRadius: BorderRadius.circular(18),
                border: Border.all(color: VeyraColors.border),
              ),
            ),
            const SizedBox(height: 14),
          ],
        ],
      );
}
