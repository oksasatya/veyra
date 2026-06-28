import 'package:decimal/decimal.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:veyra_mobile/core/error/failure.dart';
import 'package:veyra_mobile/core/error/failure_l10n.dart';
import 'package:veyra_mobile/core/theme/app_theme.dart';
import 'package:veyra_mobile/core/widgets/app_background.dart';
import 'package:veyra_mobile/core/widgets/skeleton.dart';
import 'package:veyra_mobile/core/widgets/veyra_mark.dart';
import 'package:veyra_mobile/features/auth/presentation/controllers/auth_controller.dart';
import 'package:veyra_mobile/features/document/presentation/screens/documents_overview.dart';
import 'package:veyra_mobile/features/reminder/presentation/screens/reminders_overview.dart';
import 'package:veyra_mobile/features/settings/presentation/screens/settings_screen.dart';
import 'package:veyra_mobile/features/vehicle/presentation/controllers/garage_dashboard_controller.dart';
import 'package:veyra_mobile/l10n/app_localizations.dart';

class GarageScreen extends ConsumerStatefulWidget {
  const GarageScreen({super.key});

  @override
  ConsumerState<GarageScreen> createState() => _GarageScreenState();
}

class _GarageScreenState extends ConsumerState<GarageScreen> {
  int _nav = 0;

  List<({IconData icon, String label})> _navItems(AppLocalizations l10n) => [
    (icon: Icons.directions_car_filled_outlined, label: l10n.garageNavGarage),
    (icon: Icons.notifications_none_rounded, label: l10n.garageNavReminders),
    (icon: Icons.description_outlined, label: l10n.garageNavDocs),
    (icon: Icons.settings_outlined, label: l10n.garageNavSettings),
  ];

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    // The design shows the add affordance only on the empty garage; on a
    // populated list we still offer a FAB so a second vehicle can be added.
    final hasVehicles =
        ref.watch(garageDashboardProvider).asData?.value.isEmpty == false;
    return Scaffold(
      floatingActionButton: _nav == 0 && hasVehicles
          ? FloatingActionButton.extended(
              onPressed: () => context.push('/vehicles/new'),
              backgroundColor: VeyraColors.accent,
              foregroundColor: VeyraColors.bg,
              icon: const Icon(Icons.add),
              label: Text(l10n.garageAddVehicle),
            )
          : null,
      bottomNavigationBar: _BottomNav(
        index: _nav,
        items: _navItems(l10n),
        onTap: (i) => setState(() => _nav = i),
      ),
      body: AppBackground(
        child: SafeArea(
          bottom: false,
          child: switch (_nav) {
            0 => const _DashboardTab(),
            1 => const RemindersOverview(),
            2 => const DocumentsOverview(),
            _ => const SettingsScreen(),
          },
        ),
      ),
    );
  }
}

// ── Garage dashboard tab ─────────────────────────────────────────────────────

class _DashboardTab extends ConsumerWidget {
  const _DashboardTab();

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final dashboard = ref.watch(garageDashboardProvider);
    final name = ref.watch(authControllerProvider).asData?.value?.name ?? '';
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Padding(
          padding: const EdgeInsets.fromLTRB(20, 16, 20, 8),
          child: Row(
            children: [
              Expanded(child: Text(l10n.garageTitle, style: soraDisplay(size: 32))),
              _Avatar(name: name),
            ],
          ),
        ),
        Expanded(
          child: dashboard.when(
            loading: () => const _DashboardSkeleton(),
            error: (e, _) => _ErrorState(
              message: e is Failure
                  ? localizedFailure(l10n, e)
                  : l10n.errorServer,
              onRetry: () => ref.invalidate(garageDashboardProvider),
            ),
            data: (d) => d.isEmpty
                ? _EmptyGarage(onAdd: () => context.push('/vehicles/new'))
                : RefreshIndicator(
                    color: VeyraColors.accent,
                    backgroundColor: VeyraColors.surface,
                    onRefresh: () =>
                        ref.refresh(garageDashboardProvider.future),
                    child: ListView(
                      padding: const EdgeInsets.fromLTRB(20, 4, 20, 96),
                      children: [
                        _OverviewRow(dashboard: d),
                        const SizedBox(height: 18),
                        for (final e in d.entries) ...[
                          _VehicleCard(
                            entry: e,
                            onTap: () => context.push(
                              '/vehicles/${e.vehicle.id}',
                              extra: e.vehicle,
                            ),
                          ),
                          const SizedBox(height: 14),
                        ],
                      ],
                    ),
                  ),
          ),
        ),
      ],
    );
  }
}

class _Avatar extends StatelessWidget {
  const _Avatar({required this.name});
  final String name;

  @override
  Widget build(BuildContext context) {
    final initial = name.isEmpty ? '·' : name.characters.first.toUpperCase();
    return Container(
      width: 38,
      height: 38,
      decoration: BoxDecoration(
        color: VeyraColors.surface2,
        shape: BoxShape.circle,
        border: Border.all(color: VeyraColors.border),
      ),
      alignment: Alignment.center,
      child: Text(
        initial,
        style: const TextStyle(
          color: VeyraColors.accent,
          fontWeight: FontWeight.w600,
          fontSize: 15,
        ),
      ),
    );
  }
}

class _OverviewRow extends StatelessWidget {
  const _OverviewRow({required this.dashboard});
  final GarageDashboard dashboard;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    return Row(
      children: [
        _OvTile(label: l10n.garageOverviewVehicles, value: '${dashboard.vehicleCount}'),
        const SizedBox(width: 10),
        _OvTile(
          label: l10n.garageOverviewDueSoon,
          value: '${dashboard.dueSoon}',
          accent: dashboard.dueSoon > 0,
        ),
        const SizedBox(width: 10),
        _OvTile(label: l10n.garageOverviewSpent, value: _compactRp(dashboard.totalSpent)),
      ],
    );
  }
}

class _OvTile extends StatelessWidget {
  const _OvTile({
    required this.label,
    required this.value,
    this.accent = false,
  });
  final String label;
  final String value;
  final bool accent;

  @override
  Widget build(BuildContext context) => Expanded(
    child: Container(
      padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 13),
      decoration: BoxDecoration(
        color: VeyraColors.surface,
        borderRadius: BorderRadius.circular(14),
        border: Border.all(color: VeyraColors.border),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            label,
            style: const TextStyle(color: VeyraColors.textMuted, fontSize: 12),
          ),
          const SizedBox(height: 6),
          Text(
            value,
            style: soraDisplay(
              size: 18,
              color: accent ? VeyraColors.accent : VeyraColors.text,
            ),
          ),
        ],
      ),
    ),
  );
}

class _VehicleCard extends StatelessWidget {
  const _VehicleCard({required this.entry, required this.onTap});
  final GarageEntry entry;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final v = entry.vehicle;
    final s = entry.summary;
    return Material(
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
                        Text(v.displayName, style: soraDisplay(size: 17)),
                        const SizedBox(height: 3),
                        Text(
                          '${v.plateNumber} · ${v.year}',
                          style: plexMono(size: 13),
                        ),
                      ],
                    ),
                  ),
                  if (s.upcomingReminders > 0)
                    _DueBadge(count: s.upcomingReminders),
                ],
              ),
              const SizedBox(height: 13),
              Text(
                l10n.garageCardOdometer(_grouped(v.odometer)),
                style: const TextStyle(
                  color: VeyraColors.textMuted,
                  fontSize: 13,
                ),
              ),
              const SizedBox(height: 12),
              Container(
                padding: const EdgeInsets.only(top: 12),
                decoration: const BoxDecoration(
                  border: Border(top: BorderSide(color: VeyraColors.border)),
                ),
                child: Row(
                  children: [
                    _StatItem(label: l10n.commonServices, value: '${s.totalServices}'),
                    _StatDivider(),
                    _StatItem(
                      label: l10n.garageCardFuel,
                      value: _compactRp(s.totalFuelCost),
                      color: VeyraColors.info,
                    ),
                    _StatDivider(),
                    _StatItem(
                      label: l10n.commonExpenses,
                      value: _compactRp(s.totalExpenses),
                    ),
                  ],
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class _DueBadge extends StatelessWidget {
  const _DueBadge({required this.count});
  final int count;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 5),
      decoration: BoxDecoration(
        color: const Color(0x24F26A21),
        borderRadius: BorderRadius.circular(999),
        border: Border.all(color: const Color(0x52F26A21)),
      ),
      child: Text(
        l10n.garageDueBadge(count),
        style: const TextStyle(
          color: VeyraColors.accent,
          fontSize: 12,
          fontWeight: FontWeight.w600,
        ),
      ),
    );
  }
}

class _StatItem extends StatelessWidget {
  const _StatItem({required this.label, required this.value, this.color});
  final String label;
  final String value;
  final Color? color;

  @override
  Widget build(BuildContext context) => Expanded(
    child: Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          label,
          style: const TextStyle(color: VeyraColors.textMuted, fontSize: 11),
        ),
        const SizedBox(height: 4),
        Text(
          value,
          style: TextStyle(
            color: color ?? VeyraColors.text,
            fontSize: 15,
            fontWeight: FontWeight.w600,
          ),
        ),
      ],
    ),
  );
}

class _StatDivider extends StatelessWidget {
  @override
  Widget build(BuildContext context) => Container(
    width: 1,
    height: 30,
    margin: const EdgeInsets.symmetric(horizontal: 12),
    color: VeyraColors.border,
  );
}

// ── Bottom navigation ────────────────────────────────────────────────────────

class _BottomNav extends StatelessWidget {
  const _BottomNav({
    required this.index,
    required this.items,
    required this.onTap,
  });
  final int index;
  final List<({IconData icon, String label})> items;
  final ValueChanged<int> onTap;

  @override
  Widget build(BuildContext context) => DecoratedBox(
    decoration: const BoxDecoration(
      color: VeyraColors.surface,
      border: Border(top: BorderSide(color: VeyraColors.border)),
    ),
    child: SafeArea(
      top: false,
      child: SizedBox(
        height: 62,
        child: Row(
          children: [
            for (var i = 0; i < items.length; i++)
              Expanded(
                child: InkWell(
                  onTap: () => onTap(i),
                  child: Column(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      Icon(
                        items[i].icon,
                        size: 22,
                        color: i == index
                            ? VeyraColors.accent
                            : VeyraColors.textMuted,
                      ),
                      const SizedBox(height: 4),
                      Text(
                        items[i].label,
                        style: TextStyle(
                          fontSize: 11,
                          fontWeight: FontWeight.w500,
                          color: i == index
                              ? VeyraColors.accent
                              : VeyraColors.textMuted,
                        ),
                      ),
                    ],
                  ),
                ),
              ),
          ],
        ),
      ),
    ),
  );
}

// ── States ───────────────────────────────────────────────────────────────────

class _EmptyGarage extends StatelessWidget {
  const _EmptyGarage({required this.onAdd});
  final VoidCallback onAdd;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    return Center(
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 40),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            DecoratedBox(
              decoration: BoxDecoration(
                borderRadius: BorderRadius.circular(28),
                boxShadow: [
                  BoxShadow(
                    color: VeyraColors.accent.withValues(alpha: 0.18),
                    blurRadius: 60,
                  ),
                ],
              ),
              child: Container(
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
            ),
            const SizedBox(height: 26),
            Text(l10n.garageEmptyTitle, style: soraDisplay(size: 21)),
            const SizedBox(height: 10),
            Text(
              l10n.garageEmptyBody,
              textAlign: TextAlign.center,
              style: const TextStyle(
                color: VeyraColors.textMuted,
                fontSize: 15,
                height: 1.55,
              ),
            ),
            const SizedBox(height: 26),
            FilledButton.icon(
              onPressed: onAdd,
              icon: const Icon(Icons.add, size: 20),
              label: Text(l10n.garageAddVehicle),
              style: FilledButton.styleFrom(
                minimumSize: const Size(0, 52),
                padding: const EdgeInsets.symmetric(horizontal: 26),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class _ErrorState extends StatelessWidget {
  const _ErrorState({required this.message, required this.onRetry});
  final String message;
  final VoidCallback onRetry;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    return Center(
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 36),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Container(
              width: 96,
              height: 96,
              decoration: BoxDecoration(
                color: VeyraColors.surface,
                borderRadius: BorderRadius.circular(26),
                border: Border.all(color: VeyraColors.border),
              ),
              alignment: Alignment.center,
              child: const Icon(
                Icons.cloud_off_outlined,
                color: VeyraColors.danger,
                size: 40,
              ),
            ),
            const SizedBox(height: 24),
            Text(l10n.garageErrorTitle, style: soraDisplay(size: 21)),
            const SizedBox(height: 10),
            Text(
              l10n.garageErrorBody,
              textAlign: TextAlign.center,
              style: const TextStyle(
                color: VeyraColors.textMuted,
                fontSize: 15,
                height: 1.55,
              ),
            ),
            const SizedBox(height: 16),
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 13, vertical: 9),
              decoration: BoxDecoration(
                color: VeyraColors.surface2,
                borderRadius: BorderRadius.circular(10),
                border: Border.all(color: VeyraColors.border),
              ),
              child: Text(message, style: plexMono(size: 12)),
            ),
            const SizedBox(height: 26),
            SizedBox(
              width: 280,
              child: Column(
                children: [
                  FilledButton.icon(
                    onPressed: onRetry,
                    icon: const Icon(Icons.refresh, size: 18),
                    label: Text(l10n.garageErrorTryAgain),
                    style: FilledButton.styleFrom(
                      minimumSize: const Size.fromHeight(52),
                    ),
                  ),
                  const SizedBox(height: 11),
                  SizedBox(
                    height: 50,
                    child: OutlinedButton(
                      onPressed: () {},
                      style: OutlinedButton.styleFrom(
                        foregroundColor: VeyraColors.text,
                        side: const BorderSide(color: VeyraColors.border),
                        shape: RoundedRectangleBorder(
                          borderRadius: BorderRadius.circular(14),
                        ),
                      ),
                      child: Text(l10n.garageErrorCheckSettings),
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

class _DashboardSkeleton extends StatelessWidget {
  const _DashboardSkeleton();

  @override
  Widget build(BuildContext context) => ListView(
    padding: const EdgeInsets.fromLTRB(20, 4, 20, 20),
    children: [
      Row(
        children: [
          for (var i = 0; i < 3; i++) ...[
            const Expanded(child: _OverviewSkeleton()),
            if (i < 2) const SizedBox(width: 10),
          ],
        ],
      ),
      const SizedBox(height: 18),
      for (var i = 0; i < 2; i++) ...[
        const _CardSkeleton(),
        const SizedBox(height: 14),
      ],
    ],
  );
}

class _OverviewSkeleton extends StatelessWidget {
  const _OverviewSkeleton();

  @override
  Widget build(BuildContext context) => Container(
    height: 70,
    padding: const EdgeInsets.all(13),
    decoration: BoxDecoration(
      color: VeyraColors.surface,
      borderRadius: BorderRadius.circular(14),
      border: Border.all(color: VeyraColors.border),
    ),
    child: const Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        SkeletonBox(width: 46, height: 11),
        SizedBox(height: 10),
        SkeletonBox(width: 28, height: 18),
      ],
    ),
  );
}

class _CardSkeleton extends StatelessWidget {
  const _CardSkeleton();

  @override
  Widget build(BuildContext context) => Container(
    padding: const EdgeInsets.all(16),
    decoration: BoxDecoration(
      color: VeyraColors.surface,
      borderRadius: BorderRadius.circular(18),
      border: Border.all(color: VeyraColors.border),
    ),
    child: Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const Row(
          children: [
            SkeletonBox(width: 44, height: 44, radius: 12),
            SizedBox(width: 12),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  FractionallySizedBox(
                    widthFactor: 0.55,
                    alignment: Alignment.centerLeft,
                    child: SkeletonBox(height: 14),
                  ),
                  SizedBox(height: 9),
                  FractionallySizedBox(
                    widthFactor: 0.38,
                    alignment: Alignment.centerLeft,
                    child: SkeletonBox(height: 11),
                  ),
                ],
              ),
            ),
          ],
        ),
        const SizedBox(height: 16),
        const FractionallySizedBox(
          widthFactor: 0.75,
          alignment: Alignment.centerLeft,
          child: SkeletonBox(height: 11),
        ),
        const SizedBox(height: 14),
        Padding(
          padding: const EdgeInsets.only(top: 14),
          child: DecoratedBox(
            decoration: const BoxDecoration(
              border: Border(top: BorderSide(color: VeyraColors.border)),
            ),
            child: Padding(
              padding: const EdgeInsets.only(top: 14),
              child: Row(
                children: [
                  for (var i = 0; i < 3; i++) ...[
                    const Expanded(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          SkeletonBox(width: 44, height: 10),
                          SizedBox(height: 8),
                          SkeletonBox(width: 40, height: 14),
                        ],
                      ),
                    ),
                    if (i < 2) const SizedBox(width: 14),
                  ],
                ],
              ),
            ),
          ),
        ),
      ],
    ),
  );
}

// ── Formatters ────────────────────────────────────────────────────────────────

String _grouped(int n) {
  final s = n.toString();
  final buf = StringBuffer();
  for (var i = 0; i < s.length; i++) {
    if (i > 0 && (s.length - i) % 3 == 0) buf.write(',');
    buf.write(s[i]);
  }
  return buf.toString();
}

/// Compact rupiah for tiles/stats: Rp 8.4M, Rp 920k, Rp 500. Display-only
/// compaction (the source value stays a Decimal).
String _compactRp(Decimal d) {
  final v = d.round().toBigInt().toDouble();
  if (v >= 1e9) return 'Rp ${(v / 1e9).toStringAsFixed(1)}B';
  if (v >= 1e6) return 'Rp ${(v / 1e6).toStringAsFixed(1)}M';
  if (v >= 1e3) return 'Rp ${(v / 1e3).toStringAsFixed(0)}k';
  return 'Rp ${v.toStringAsFixed(0)}';
}
