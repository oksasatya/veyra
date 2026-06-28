import 'package:decimal/decimal.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:veyra_mobile/core/error/failure.dart';
import 'package:veyra_mobile/core/theme/app_theme.dart';
import 'package:veyra_mobile/features/vehicle/data/repositories/vehicle_repository_impl.dart';
import 'package:veyra_mobile/features/vehicle/domain/entities/vehicle.dart';
import 'package:veyra_mobile/features/vehicle/domain/entities/vehicle_summary.dart';

const _tabs = ['Overview', 'Fuel', 'Service', 'Expenses', 'Docs'];

class VehicleDetailScreen extends ConsumerWidget {
  const VehicleDetailScreen({required this.vehicle, super.key});
  final Vehicle vehicle;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final summary = ref.watch(vehicleSummaryProvider(vehicle.id));
    return Scaffold(
      appBar: AppBar(title: Text(vehicle.displayName, style: soraDisplay(size: 18))),
      bottomNavigationBar: const _LogBar(),
      body: SafeArea(
        top: false,
        child: ListView(
          padding: const EdgeInsets.fromLTRB(20, 8, 20, 24),
          children: [
            Text(vehicle.displayName, style: soraDisplay(size: 26)),
            const SizedBox(height: 6),
            Text(
              '${vehicle.plateNumber} · ${vehicle.year} · ${vehicle.fuelType.label}'
              '${vehicle.color != null ? ' · ${vehicle.color}' : ''}',
              style: plexMono(size: 13),
            ),
            const SizedBox(height: 16),
            _OdometerCard(vehicle: vehicle, summary: summary.asData?.value),
            const SizedBox(height: 16),
            summary.when(
              loading: () => const _StatsSkeleton(),
              error: (e, _) => _StatsError(
                message: e is Failure ? e.message : 'Could not load summary.',
                onRetry: () => ref.invalidate(vehicleSummaryProvider(vehicle.id)),
              ),
              data: (s) => _StatsGrid(summary: s),
            ),
            const SizedBox(height: 18),
            const _TabRow(),
            const SizedBox(height: 20),
            const _ActivityPlaceholder(),
          ],
        ),
      ),
    );
  }
}

class _OdometerCard extends StatelessWidget {
  const _OdometerCard({required this.vehicle, required this.summary});
  final Vehicle vehicle;
  final VehicleSummary? summary;

  @override
  Widget build(BuildContext context) {
    final due = summary?.upcomingReminders ?? 0;
    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: VeyraColors.surface,
        borderRadius: BorderRadius.circular(14),
        border: Border.all(color: VeyraColors.border),
      ),
      child: Row(
        children: [
          Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              const Text('Odometer',
                  style: TextStyle(color: VeyraColors.textMuted, fontSize: 12)),
              const SizedBox(height: 4),
              Text('${_grouped(vehicle.odometer)} km', style: plexMono(size: 20, color: VeyraColors.text)),
            ],
          ),
          const Spacer(),
          Column(
            crossAxisAlignment: CrossAxisAlignment.end,
            children: [
              const Text('Due soon',
                  style: TextStyle(color: VeyraColors.accent, fontSize: 12)),
              const SizedBox(height: 4),
              Text('$due reminder${due == 1 ? '' : 's'}',
                  style: const TextStyle(color: VeyraColors.text, fontSize: 14)),
            ],
          ),
        ],
      ),
    );
  }
}

class _StatsGrid extends StatelessWidget {
  const _StatsGrid({required this.summary});
  final VehicleSummary summary;

  @override
  Widget build(BuildContext context) => GridView.count(
        crossAxisCount: 2,
        shrinkWrap: true,
        physics: const NeverScrollableScrollPhysics(),
        mainAxisSpacing: 12,
        crossAxisSpacing: 12,
        childAspectRatio: 2.0,
        children: [
          _StatCell(label: 'Services', value: '${summary.totalServices}'),
          _StatCell(
            label: 'Service cost',
            value: _money(summary.totalServiceCost),
            color: VeyraColors.accent,
          ),
          _StatCell(label: 'Refuels', value: '${summary.totalRefuels}'),
          _StatCell(
            label: 'Fuel cost',
            value: _money(summary.totalFuelCost),
            color: VeyraColors.info,
          ),
        ],
      );
}

class _StatCell extends StatelessWidget {
  const _StatCell({required this.label, required this.value, this.color});
  final String label;
  final String value;
  final Color? color;

  @override
  Widget build(BuildContext context) => Container(
        padding: const EdgeInsets.symmetric(horizontal: 15, vertical: 12),
        decoration: BoxDecoration(
          color: VeyraColors.surface,
          borderRadius: BorderRadius.circular(14),
          border: Border.all(color: VeyraColors.border),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Text(label.toUpperCase(),
                style: const TextStyle(
                    color: VeyraColors.textMuted, fontSize: 11, letterSpacing: 0.4)),
            const SizedBox(height: 6),
            Text(value, style: soraDisplay(size: 18, color: color ?? VeyraColors.text)),
          ],
        ),
      );
}

class _TabRow extends StatelessWidget {
  const _TabRow();

  @override
  Widget build(BuildContext context) => SizedBox(
        height: 38,
        child: ListView.separated(
          scrollDirection: Axis.horizontal,
          itemCount: _tabs.length,
          separatorBuilder: (_, _) => const SizedBox(width: 8),
          itemBuilder: (context, i) {
            final active = i == 0;
            return Container(
              padding: const EdgeInsets.symmetric(horizontal: 16),
              alignment: Alignment.center,
              decoration: BoxDecoration(
                color: active ? VeyraColors.accent : VeyraColors.surface,
                borderRadius: BorderRadius.circular(999),
                border: Border.all(
                    color: active ? VeyraColors.accent : VeyraColors.border),
              ),
              child: Text(
                _tabs[i],
                style: TextStyle(
                  color: active ? VeyraColors.bg : VeyraColors.textMuted,
                  fontSize: 14,
                  fontWeight: FontWeight.w500,
                ),
              ),
            );
          },
        ),
      );
}

class _ActivityPlaceholder extends StatelessWidget {
  const _ActivityPlaceholder();

  @override
  Widget build(BuildContext context) => Container(
        padding: const EdgeInsets.symmetric(vertical: 28),
        alignment: Alignment.center,
        child: const Text(
          'Service, fuel & expense history arrives next.',
          style: TextStyle(color: VeyraColors.textMuted, fontSize: 14),
        ),
      );
}

class _StatsSkeleton extends StatelessWidget {
  const _StatsSkeleton();

  @override
  Widget build(BuildContext context) => GridView.count(
        crossAxisCount: 2,
        shrinkWrap: true,
        physics: const NeverScrollableScrollPhysics(),
        mainAxisSpacing: 12,
        crossAxisSpacing: 12,
        childAspectRatio: 2.0,
        children: [
          for (var i = 0; i < 4; i++)
            DecoratedBox(
              decoration: BoxDecoration(
                color: VeyraColors.surface,
                borderRadius: BorderRadius.circular(14),
                border: Border.all(color: VeyraColors.border),
              ),
            ),
        ],
      );
}

class _StatsError extends StatelessWidget {
  const _StatsError({required this.message, required this.onRetry});
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
              child: Text(message,
                  style: const TextStyle(color: VeyraColors.textMuted, fontSize: 13)),
            ),
            TextButton(onPressed: onRetry, child: const Text('Retry')),
          ],
        ),
      );
}

class _LogBar extends StatelessWidget {
  const _LogBar();

  @override
  Widget build(BuildContext context) => const SafeArea(
        child: Padding(
          padding: EdgeInsets.fromLTRB(20, 8, 20, 8),
          child: FilledButton(
            onPressed: null,
            child: Text('Log fuel'),
          ),
        ),
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
