import 'package:decimal/decimal.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:veyra_mobile/core/error/failure.dart';
import 'package:veyra_mobile/core/error/failure_l10n.dart';
import 'package:veyra_mobile/core/theme/app_theme.dart';
import 'package:veyra_mobile/features/fuel_log/data/repositories/fuel_log_repository_impl.dart';
import 'package:veyra_mobile/features/fuel_log/domain/entities/fuel_log.dart';
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

/// Renders the fuel logs for [vehicleId] with loading / empty / error states.
class FuelLogList extends ConsumerWidget {
  const FuelLogList({required this.vehicleId, super.key});
  final String vehicleId;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final logs = ref.watch(fuelLogListProvider(vehicleId));
    return logs.when(
      loading: () => const _FuelLogSkeleton(),
      error: (e, _) => _FuelLogError(
        message: e is Failure
            ? localizedFailure(l10n, e)
            : l10n.fuelLogLoadError,
        onRetry: () => ref.invalidate(fuelLogListProvider(vehicleId)),
      ),
      data: (rows) => rows.isEmpty
          ? _FuelLogEmpty(l10n: l10n)
          : Column(
              children: [
                for (final log in rows) _FuelLogRow(log: log),
              ],
            ),
    );
  }
}

class _FuelLogRow extends StatelessWidget {
  const _FuelLogRow({required this.log});
  final FuelLog log;

  @override
  Widget build(BuildContext context) {
    final station = log.station;
    final meta =
        '${_grouped(log.odometer)} km · '
        '${log.liters} L${station != null && station.isNotEmpty ? ' · $station' : ''}';
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
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  _formatDate(log.logDate),
                  style: const TextStyle(
                    color: VeyraColors.text,
                    fontSize: 15,
                    fontWeight: FontWeight.w500,
                  ),
                ),
                const SizedBox(height: 4),
                Text(meta, style: plexMono(size: 12)),
              ],
            ),
          ),
          const SizedBox(width: 12),
          Text(
            _money(log.totalCost),
            style: plexMono(size: 15, color: VeyraColors.accent),
          ),
        ],
      ),
    );
  }
}

class _FuelLogEmpty extends StatelessWidget {
  const _FuelLogEmpty({required this.l10n});
  final AppLocalizations l10n;

  @override
  Widget build(BuildContext context) => Container(
    padding: const EdgeInsets.symmetric(vertical: 36, horizontal: 20),
    alignment: Alignment.center,
    child: Text(
      l10n.fuelLogEmpty,
      textAlign: TextAlign.center,
      style: const TextStyle(color: VeyraColors.textMuted, fontSize: 14),
    ),
  );
}

class _FuelLogSkeleton extends StatelessWidget {
  const _FuelLogSkeleton();

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

class _FuelLogError extends StatelessWidget {
  const _FuelLogError({required this.message, required this.onRetry});
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
              style: const TextStyle(
                color: VeyraColors.textMuted,
                fontSize: 13,
              ),
            ),
          ),
          TextButton(onPressed: onRetry, child: Text(l10n.commonRetry)),
        ],
      ),
    );
  }
}

String _formatDate(DateTime d) => '${d.day} ${_months[d.month - 1]} ${d.year}';

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
