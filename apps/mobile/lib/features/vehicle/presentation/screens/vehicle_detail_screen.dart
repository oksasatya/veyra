import 'package:decimal/decimal.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:veyra_mobile/core/error/failure.dart';
import 'package:veyra_mobile/core/error/failure_l10n.dart';
import 'package:veyra_mobile/core/theme/app_theme.dart';
import 'package:veyra_mobile/core/widgets/app_background.dart';
import 'package:veyra_mobile/core/widgets/segmented_tabs.dart';
import 'package:veyra_mobile/features/document/presentation/widgets/add_document_sheet.dart';
import 'package:veyra_mobile/features/document/presentation/widgets/document_list.dart';
import 'package:veyra_mobile/features/expense/presentation/expense_l10n.dart';
import 'package:veyra_mobile/features/expense/presentation/widgets/add_expense_sheet.dart';
import 'package:veyra_mobile/features/expense/presentation/widgets/expense_list.dart';
import 'package:veyra_mobile/features/fuel_log/presentation/widgets/add_fuel_log_sheet.dart';
import 'package:veyra_mobile/features/fuel_log/presentation/widgets/fuel_log_list.dart';
import 'package:veyra_mobile/features/service_record/presentation/widgets/add_service_record_sheet.dart';
import 'package:veyra_mobile/features/service_record/presentation/widgets/service_record_list.dart';
import 'package:veyra_mobile/features/vehicle/data/repositories/vehicle_repository_impl.dart';
import 'package:veyra_mobile/features/vehicle/domain/entities/vehicle.dart';
import 'package:veyra_mobile/features/vehicle/domain/entities/vehicle_summary.dart';
import 'package:veyra_mobile/features/vehicle/presentation/vehicle_activity.dart';
import 'package:veyra_mobile/features/vehicle/presentation/vehicle_l10n.dart';
import 'package:veyra_mobile/l10n/app_localizations.dart';

class VehicleDetailScreen extends ConsumerStatefulWidget {
  const VehicleDetailScreen({required this.vehicle, super.key});
  final Vehicle vehicle;

  @override
  ConsumerState<VehicleDetailScreen> createState() =>
      _VehicleDetailScreenState();
}

class _VehicleDetailScreenState extends ConsumerState<VehicleDetailScreen> {
  int _tab = 0;

  String get _vehicleId => widget.vehicle.id;

  String _addLabel(AppLocalizations l10n) => switch (_tab) {
    2 => l10n.vehicleDetailAddService,
    3 => l10n.vehicleDetailAddExpense,
    4 => l10n.vehicleDetailAddDocument,
    _ => l10n.vehicleDetailAddFuel,
  };

  Future<void> _openAdd() async {
    Widget sheet() => switch (_tab) {
      2 => AddServiceRecordSheet(vehicleId: _vehicleId),
      3 => AddExpenseSheet(vehicleId: _vehicleId),
      4 => AddDocumentSheet(vehicleId: _vehicleId),
      _ => AddFuelLogSheet(vehicleId: _vehicleId),
    };
    await showModalBottomSheet<void>(
      context: context,
      isScrollControlled: true,
      backgroundColor: Colors.transparent,
      builder: (_) => sheet(),
    );
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final vehicle = widget.vehicle;
    final summary = ref.watch(vehicleSummaryProvider(_vehicleId));
    final tabs = [
      l10n.vehicleDetailTabOverview,
      l10n.vehicleDetailTabFuel,
      l10n.vehicleDetailTabService,
      l10n.vehicleDetailTabExpenses,
      l10n.vehicleDetailTabDocs,
    ];
    return Scaffold(
      appBar: AppBar(
        title: Text(vehicle.displayName, style: soraDisplay(size: 18)),
      ),
      bottomNavigationBar: _AddBar(label: _addLabel(l10n), onPressed: _openAdd),
      body: AppBackground(
        child: SafeArea(
          top: false,
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              Padding(
                padding: const EdgeInsets.fromLTRB(20, 16, 20, 0),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(vehicle.displayName, style: soraDisplay(size: 24)),
                    const SizedBox(height: 6),
                    Text(
                      '${vehicle.plateNumber} · ${vehicle.year} · ${localizedFuelType(l10n, vehicle.fuelType)}'
                      '${vehicle.color != null ? ' · ${vehicle.color}' : ''}',
                      style: plexMono(size: 13),
                    ),
                    const SizedBox(height: 14),
                    _OdometerCard(
                      vehicle: vehicle,
                      summary: summary.asData?.value,
                    ),
                    const SizedBox(height: 16),
                  ],
                ),
              ),
              SegmentedTabs(
                labels: tabs,
                index: _tab,
                onChanged: (i) => setState(() => _tab = i),
              ),
              const SizedBox(height: 14),
              Expanded(child: _content(summary)),
            ],
          ),
        ),
      ),
    );
  }

  Widget _content(AsyncValue<VehicleSummary> summary) => switch (_tab) {
    1 => FuelLogList(vehicleId: _vehicleId),
    2 => ServiceRecordList(vehicleId: _vehicleId),
    3 => ExpenseList(vehicleId: _vehicleId),
    4 => DocumentList(vehicleId: _vehicleId),
    _ => _Overview(summary: summary, vehicleId: _vehicleId),
  };
}

class _Overview extends ConsumerWidget {
  const _Overview({required this.summary, required this.vehicleId});
  final AsyncValue<VehicleSummary> summary;
  final String vehicleId;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    return ListView(
      padding: const EdgeInsets.fromLTRB(20, 0, 20, 24),
      children: [
        summary.when(
          loading: () => const _StatsSkeleton(),
          error: (e, _) => _StatsError(
            message: e is Failure
                ? localizedFailure(l10n, e)
                : l10n.errorServer,
            onRetry: () => ref.invalidate(vehicleSummaryProvider(vehicleId)),
          ),
          data: (s) => _StatsGrid(summary: s),
        ),
        const SizedBox(height: 20),
        _ActivityFeed(vehicleId: vehicleId),
      ],
    );
  }
}

class _OdometerCard extends StatelessWidget {
  const _OdometerCard({required this.vehicle, required this.summary});
  final Vehicle vehicle;
  final VehicleSummary? summary;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
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
              Text(
                l10n.vehicleDetailOdometerLabel,
                style: const TextStyle(color: VeyraColors.textMuted, fontSize: 12),
              ),
              const SizedBox(height: 4),
              Text(
                '${_grouped(vehicle.odometer)} km',
                style: plexMono(size: 20, color: VeyraColors.text),
              ),
            ],
          ),
          const Spacer(),
          Column(
            crossAxisAlignment: CrossAxisAlignment.end,
            children: [
              Text(
                l10n.vehicleDetailDueSoon,
                style: const TextStyle(color: VeyraColors.accent, fontSize: 12),
              ),
              const SizedBox(height: 4),
              Text(
                l10n.vehicleDetailReminders(due),
                style: const TextStyle(color: VeyraColors.text, fontSize: 14),
              ),
            ],
          ),
        ],
      ),
    );
  }
}

/// 2x2 stat grid as a single connected card with 1px hairline dividers
/// (design `.grid` — border bg showing through 1px gaps, rounded + clipped).
class _StatsGrid extends StatelessWidget {
  const _StatsGrid({required this.summary});
  final VehicleSummary summary;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    return ClipRRect(
      borderRadius: BorderRadius.circular(14),
      child: DecoratedBox(
        decoration: BoxDecoration(
          border: Border.all(color: VeyraColors.border),
          borderRadius: BorderRadius.circular(14),
        ),
        child: Column(
          children: [
            IntrinsicHeight(
              child: Row(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  Expanded(
                    child: _StatCell(
                      label: l10n.vehicleDetailStatServices,
                      value: '${summary.totalServices}',
                    ),
                  ),
                  const _VDivider(),
                  Expanded(
                    child: _StatCell(
                      label: l10n.vehicleDetailStatServiceCost,
                      value: _money(summary.totalServiceCost),
                      color: VeyraColors.accent,
                    ),
                  ),
                ],
              ),
            ),
            const _HDivider(),
            IntrinsicHeight(
              child: Row(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  Expanded(
                    child: _StatCell(
                      label: l10n.vehicleDetailStatRefuels,
                      value: '${summary.totalRefuels}',
                    ),
                  ),
                  const _VDivider(),
                  Expanded(
                    child: _StatCell(
                      label: l10n.vehicleDetailStatFuelCost,
                      value: _money(summary.totalFuelCost),
                      color: VeyraColors.info,
                    ),
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class _VDivider extends StatelessWidget {
  const _VDivider();
  @override
  Widget build(BuildContext context) =>
      const ColoredBox(color: VeyraColors.border, child: SizedBox(width: 1));
}

class _HDivider extends StatelessWidget {
  const _HDivider();
  @override
  Widget build(BuildContext context) =>
      const ColoredBox(color: VeyraColors.border, child: SizedBox(height: 1));
}

class _StatCell extends StatelessWidget {
  const _StatCell({required this.label, required this.value, this.color});
  final String label;
  final String value;
  final Color? color;

  @override
  Widget build(BuildContext context) => ColoredBox(
    color: VeyraColors.surface,
    child: Padding(
      padding: const EdgeInsets.symmetric(horizontal: 15, vertical: 14),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            label.toUpperCase(),
            style: const TextStyle(
              color: VeyraColors.textMuted,
              fontSize: 11,
              letterSpacing: 0.4,
            ),
          ),
          const SizedBox(height: 6),
          Text(
            value,
            style: soraDisplay(size: 18, color: color ?? VeyraColors.text),
          ),
        ],
      ),
    ),
  );
}

/// Cross-type recent activity (fuel + service + expense) for the Overview tab.
class _ActivityFeed extends ConsumerWidget {
  const _ActivityFeed({required this.vehicleId});
  final String vehicleId;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final activity = ref.watch(vehicleActivityProvider(vehicleId));
    return activity.when(
      loading: () => const Padding(
        padding: EdgeInsets.symmetric(vertical: 24),
        child: Center(
          child: SizedBox(
            width: 22,
            height: 22,
            child: CircularProgressIndicator(
              strokeWidth: 2.2,
              color: VeyraColors.accent,
            ),
          ),
        ),
      ),
      error: (_, _) => _ActivityNote(l10n.errorServer),
      data: (items) => items.isEmpty
          ? _ActivityNote(l10n.vehicleActivityEmpty)
          : Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(l10n.vehicleActivityTitle, style: soraDisplay(size: 16)),
                const SizedBox(height: 6),
                for (final item in items) _ActivityRow(item: item),
              ],
            ),
    );
  }
}

class _ActivityNote extends StatelessWidget {
  const _ActivityNote(this.text);
  final String text;

  @override
  Widget build(BuildContext context) => Container(
    padding: const EdgeInsets.symmetric(vertical: 24),
    alignment: Alignment.center,
    child: Text(
      text,
      textAlign: TextAlign.center,
      style: const TextStyle(color: VeyraColors.textMuted, fontSize: 14),
    ),
  );
}

class _ActivityRow extends StatelessWidget {
  const _ActivityRow({required this.item});
  final ActivityItem item;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final v = _present(l10n, item);
    return Container(
      padding: const EdgeInsets.symmetric(vertical: 13),
      decoration: const BoxDecoration(
        border: Border(bottom: BorderSide(color: VeyraColors.border)),
      ),
      child: Row(
        children: [
          Container(
            width: 40,
            height: 40,
            decoration: BoxDecoration(
              color: VeyraColors.surface2,
              borderRadius: BorderRadius.circular(11),
              border: Border.all(color: VeyraColors.border),
            ),
            child: Icon(v.icon, size: 20, color: v.color),
          ),
          const SizedBox(width: 13),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  v.title,
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                  style: const TextStyle(
                    color: VeyraColors.text,
                    fontSize: 15,
                    fontWeight: FontWeight.w500,
                  ),
                ),
                const SizedBox(height: 3),
                Text(
                  v.meta,
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                  style: plexMono(size: 12),
                ),
              ],
            ),
          ),
          if (v.amount != null) ...[
            const SizedBox(width: 10),
            Text(
              v.amount!,
              style: plexMono(
                size: 14,
                color: v.amountAccent ? VeyraColors.accent : VeyraColors.text,
              ),
            ),
          ],
        ],
      ),
    );
  }
}

typedef _Presented = ({
  IconData icon,
  Color color,
  String title,
  String meta,
  String? amount,
  bool amountAccent,
});

_Presented _present(AppLocalizations l10n, ActivityItem item) => switch (item) {
  FuelActivity(:final log) => (
    icon: Icons.local_gas_station_outlined,
    color: VeyraColors.accent,
    title: '${l10n.vehicleDetailTabFuel} · ${log.station ?? '—'}',
    meta:
        '${_shortDate(log.logDate)} · ${_grouped(log.odometer)} km · ${log.liters} L',
    amount: _money(log.totalCost),
    amountAccent: true,
  ),
  ServiceActivity(:final record) => (
    icon: Icons.build_outlined,
    color: VeyraColors.textMuted,
    title: '${l10n.vehicleDetailTabService} · ${record.workshop ?? '—'}',
    meta:
        '${_shortDate(record.serviceDate)} · ${_grouped(record.odometer)} km · ${record.description}',
    amount: record.cost != null ? _money(record.cost!) : null,
    amountAccent: false,
  ),
  ExpenseActivity(:final expense) => (
    icon: Icons.payments_outlined,
    color: VeyraColors.info,
    title:
        '${l10n.vehicleActivityExpense} · ${localizedExpenseCategory(l10n, expense.category)}',
    meta: '${_shortDate(expense.expenseDate)} · ${expense.description}',
    amount: _money(expense.amount),
    amountAccent: false,
  ),
};

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

String _shortDate(DateTime d) => '${d.day} ${_months[d.month - 1]}';

class _StatsSkeleton extends StatelessWidget {
  const _StatsSkeleton();

  @override
  Widget build(BuildContext context) => GridView.count(
    crossAxisCount: 2,
    shrinkWrap: true,
    physics: const NeverScrollableScrollPhysics(),
    mainAxisSpacing: 12,
    crossAxisSpacing: 12,
    childAspectRatio: 2,
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
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    return Container(
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
              style: const TextStyle(color: VeyraColors.textMuted, fontSize: 13),
            ),
          ),
          TextButton(
            onPressed: onRetry,
            child: Text(l10n.vehicleDetailErrorRetry),
          ),
        ],
      ),
    );
  }
}

class _AddBar extends StatelessWidget {
  const _AddBar({required this.label, required this.onPressed});
  final String label;
  final VoidCallback onPressed;

  @override
  Widget build(BuildContext context) => SafeArea(
    child: Padding(
      padding: const EdgeInsets.fromLTRB(20, 8, 20, 8),
      child: FilledButton.icon(
        onPressed: onPressed,
        icon: const Icon(Icons.add, size: 20),
        label: Text(label),
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
