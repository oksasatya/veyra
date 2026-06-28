import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:veyra_mobile/core/error/failure.dart';
import 'package:veyra_mobile/core/error/failure_l10n.dart';
import 'package:veyra_mobile/core/theme/app_theme.dart';
import 'package:veyra_mobile/core/widgets/skeleton.dart';
import 'package:veyra_mobile/core/widgets/status_pill.dart';
import 'package:veyra_mobile/features/document/domain/value_objects/doc_type.dart';
import 'package:veyra_mobile/features/document/domain/value_objects/expiry_status.dart';
import 'package:veyra_mobile/features/document/presentation/controllers/documents_overview_controller.dart';
import 'package:veyra_mobile/l10n/app_localizations.dart';

/// Cross-vehicle documents with expiry status (design `documents.html`).
class DocumentsOverview extends ConsumerWidget {
  const DocumentsOverview({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final docs = ref.watch(documentsOverviewProvider);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Padding(
          padding: const EdgeInsets.fromLTRB(20, 16, 20, 4),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(l10n.documentTitle, style: soraDisplay(size: 30)),
              const SizedBox(height: 4),
              Text(
                docs.asData == null
                    ? l10n.documentAllVehicles
                    : l10n.documentCountAcrossVehicles(
                        docs.asData!.value.length,
                      ),
                style: const TextStyle(
                  color: VeyraColors.textMuted,
                  fontSize: 13,
                ),
              ),
            ],
          ),
        ),
        Expanded(
          child: docs.when(
            loading: () => const _DocsSkeleton(),
            error: (e, _) => _SectionError(
              message: e is Failure
                  ? localizedFailure(l10n, e)
                  : l10n.documentErrorTitle,
              onRetry: () => ref.invalidate(documentsOverviewProvider),
            ),
            data: (items) => items.isEmpty
                ? const _DocsEmpty()
                : ListView(
                    padding: const EdgeInsets.fromLTRB(20, 14, 20, 96),
                    children: [for (final d in items) _DocRow(item: d)],
                  ),
          ),
        ),
      ],
    );
  }
}

class _DocRow extends StatelessWidget {
  const _DocRow({required this.item});
  final DocumentWithVehicle item;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final doc = item.document;
    final status = expiryStatusFor(
      expiry: doc.expiryDate,
      today: DateTime.now(),
    );
    return Container(
      margin: const EdgeInsets.only(bottom: 12),
      padding: const EdgeInsets.all(15),
      decoration: BoxDecoration(
        color: VeyraColors.surface,
        borderRadius: BorderRadius.circular(16),
        border: Border.all(color: VeyraColors.border),
      ),
      child: Row(
        children: [
          Container(
            width: 46,
            height: 46,
            decoration: BoxDecoration(
              color: VeyraColors.surface2,
              borderRadius: BorderRadius.circular(12),
              border: Border.all(color: VeyraColors.border),
            ),
            child: Icon(_icon(doc.docType), color: _iconColor(status), size: 22),
          ),
          const SizedBox(width: 14),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  doc.title,
                  style: const TextStyle(
                    color: VeyraColors.text,
                    fontSize: 15,
                    fontWeight: FontWeight.w600,
                  ),
                ),
                const SizedBox(height: 4),
                Text(_meta(l10n, item, status), style: plexMono(size: 12)),
              ],
            ),
          ),
          const SizedBox(width: 10),
          _pill(l10n, status, doc.expiryDate),
        ],
      ),
    );
  }
}

IconData _icon(DocType type) => switch (type) {
      DocType.stnk => Icons.description_outlined,
      DocType.bpkb => Icons.menu_book_outlined,
      DocType.insurance => Icons.shield_outlined,
      DocType.other => Icons.insert_drive_file_outlined,
    };

Color _iconColor(ExpiryStatus status) => switch (status) {
      ExpiryStatus.valid => VeyraColors.ok,
      ExpiryStatus.onFile => VeyraColors.textMuted,
      _ => VeyraColors.accent,
    };

StatusPill _pill(
  AppLocalizations l10n,
  ExpiryStatus status,
  DateTime? expiry,
) {
  switch (status) {
    case ExpiryStatus.expired:
      return StatusPill(l10n.documentStatusExpired, tone: PillTone.danger);
    case ExpiryStatus.expiringSoon:
      final days = _daysUntil(expiry!);
      return StatusPill(
        l10n.documentDaysLeft(days),
        tone: PillTone.accent,
      );
    case ExpiryStatus.valid:
      return StatusPill(l10n.documentStatusValid, tone: PillTone.ok);
    case ExpiryStatus.onFile:
      return StatusPill(l10n.documentStatusOnFile, tone: PillTone.muted);
  }
}

String _meta(
  AppLocalizations l10n,
  DocumentWithVehicle item,
  ExpiryStatus status,
) {
  final expiry = item.document.expiryDate;
  final tail = switch (status) {
    ExpiryStatus.onFile => l10n.documentNoExpiry,
    ExpiryStatus.valid => l10n.documentValidUntil(_longDate(expiry)),
    _ => l10n.documentExpires(_longDate(expiry)),
  };
  return '${item.vehicleName} · $tail';
}

class _DocsSkeleton extends StatelessWidget {
  const _DocsSkeleton();

  @override
  Widget build(BuildContext context) => ListView(
        padding: const EdgeInsets.fromLTRB(20, 14, 20, 0),
        children: [
          for (var i = 0; i < 4; i++)
            const Padding(
              padding: EdgeInsets.only(bottom: 12),
              child: SkeletonBox(height: 76, radius: 16),
            ),
        ],
      );
}

class _DocsEmpty extends StatelessWidget {
  const _DocsEmpty();

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    return Center(
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 40),
        child: Text(
          l10n.documentEmptyBody,
          textAlign: TextAlign.center,
          style: const TextStyle(color: VeyraColors.textMuted, fontSize: 15),
        ),
      ),
    );
  }
}

class _SectionError extends StatelessWidget {
  const _SectionError({required this.message, required this.onRetry});
  final String message;
  final VoidCallback onRetry;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    return Center(
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 40),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Text(
              message,
              textAlign: TextAlign.center,
              style: const TextStyle(
                color: VeyraColors.textMuted,
                fontSize: 15,
              ),
            ),
            const SizedBox(height: 16),
            FilledButton(
              onPressed: onRetry,
              child: Text(l10n.commonTryAgain),
            ),
          ],
        ),
      ),
    );
  }
}

int _daysUntil(DateTime date) {
  final now = DateTime.now();
  final today = DateTime(now.year, now.month, now.day);
  final target = DateTime(date.year, date.month, date.day);
  return target.difference(today).inDays;
}

const _months = [
  'Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun',
  'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec',
];

String _longDate(DateTime? date) =>
    date == null ? '—' : '${date.day} ${_months[date.month - 1]} ${date.year}';
