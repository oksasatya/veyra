import 'package:decimal/decimal.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:veyra_mobile/core/error/failure.dart';
import 'package:veyra_mobile/core/theme/app_theme.dart';
import 'package:veyra_mobile/features/service_record/data/repositories/service_record_repository_impl.dart';
import 'package:veyra_mobile/features/service_record/domain/entities/service_record.dart';

/// Lists a vehicle's service records (date · odometer · description · workshop,
/// trailing `Rp cost` when present) with loading / empty / error states.
class ServiceRecordList extends ConsumerWidget {
  const ServiceRecordList({required this.vehicleId, super.key});
  final String vehicleId;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final records = ref.watch(serviceRecordListProvider(vehicleId));
    return records.when(
      loading: () => const _ListSkeleton(),
      error: (e, _) => _ListError(
        message: e is Failure ? e.message : 'Could not load service records.',
        onRetry: () => ref.invalidate(serviceRecordListProvider(vehicleId)),
      ),
      data: (items) => items.isEmpty
          ? const _EmptyState()
          : Column(
              children: [
                for (final r in items) ...[
                  _ServiceRecordTile(record: r),
                  const SizedBox(height: 10),
                ],
              ],
            ),
    );
  }
}

class _ServiceRecordTile extends StatelessWidget {
  const _ServiceRecordTile({required this.record});
  final ServiceRecord record;

  @override
  Widget build(BuildContext context) {
    final meta = StringBuffer()
      ..write(_date(record.serviceDate))
      ..write(' · ${_grouped(record.odometer)} km');
    if (record.workshop != null && record.workshop!.isNotEmpty) {
      meta.write(' · ${record.workshop}');
    }
    return Container(
      padding: const EdgeInsets.all(14),
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
                  record.description,
                  style: const TextStyle(
                    color: VeyraColors.text,
                    fontSize: 15,
                    fontWeight: FontWeight.w500,
                  ),
                ),
                const SizedBox(height: 4),
                Text(meta.toString(), style: plexMono(size: 12)),
              ],
            ),
          ),
          if (record.cost != null) ...[
            const SizedBox(width: 12),
            Text(
              _money(record.cost!),
              style: plexMono(size: 14, color: VeyraColors.accent),
            ),
          ],
        ],
      ),
    );
  }
}

class _EmptyState extends StatelessWidget {
  const _EmptyState();

  @override
  Widget build(BuildContext context) => Container(
        padding: const EdgeInsets.symmetric(vertical: 28),
        alignment: Alignment.center,
        child: const Text(
          'No service records yet. Log the first one.',
          style: TextStyle(color: VeyraColors.textMuted, fontSize: 14),
        ),
      );
}

class _ListSkeleton extends StatelessWidget {
  const _ListSkeleton();

  @override
  Widget build(BuildContext context) => Column(
        children: [
          for (var i = 0; i < 3; i++) ...[
            Container(
              height: 64,
              decoration: BoxDecoration(
                color: VeyraColors.surface,
                borderRadius: BorderRadius.circular(14),
                border: Border.all(color: VeyraColors.border),
              ),
            ),
            const SizedBox(height: 10),
          ],
        ],
      );
}

class _ListError extends StatelessWidget {
  const _ListError({required this.message, required this.onRetry});
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

String _date(DateTime d) =>
    '${d.year.toString().padLeft(4, '0')}-'
    '${d.month.toString().padLeft(2, '0')}-'
    '${d.day.toString().padLeft(2, '0')}';

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
