import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:veyra_mobile/core/error/failure.dart';
import 'package:veyra_mobile/core/theme/app_theme.dart';
import 'package:veyra_mobile/features/document/data/repositories/document_repository_impl.dart';
import 'package:veyra_mobile/features/document/domain/entities/document.dart';
import 'package:veyra_mobile/features/document/domain/value_objects/expiry_status.dart';

/// Documents for a vehicle: doc icon · title · expiry meta, with an
/// expiry-status pill derived from the domain helper. Renders loading / empty /
/// error states inline.
class DocumentList extends ConsumerWidget {
  const DocumentList({required this.vehicleId, super.key});
  final String vehicleId;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final docs = ref.watch(documentListProvider(vehicleId));
    return docs.when(
      loading: () => const _DocsSkeleton(),
      error: (e, _) => _ErrorState(
        message: e is Failure ? e.message : 'Something went wrong.',
        onRetry: () => ref.invalidate(documentListProvider(vehicleId)),
      ),
      data: (list) => list.isEmpty
          ? const _EmptyDocs()
          : _DocList(
              documents: list,
              onRefresh: () =>
                  ref.refresh(documentListProvider(vehicleId).future),
            ),
    );
  }
}

class _DocList extends StatelessWidget {
  const _DocList({required this.documents, required this.onRefresh});
  final List<Document> documents;
  final Future<void> Function() onRefresh;

  @override
  Widget build(BuildContext context) {
    final today = DateTime.now();
    return RefreshIndicator(
      onRefresh: onRefresh,
      color: VeyraColors.accent,
      backgroundColor: VeyraColors.surface,
      child: ListView(
        padding: const EdgeInsets.fromLTRB(20, 8, 20, 96),
        children: [
          for (final d in documents) ...[
            _DocCard(document: d, today: today),
            const SizedBox(height: 12),
          ],
        ],
      ),
    );
  }
}

class _DocCard extends StatelessWidget {
  const _DocCard({required this.document, required this.today});
  final Document document;
  final DateTime today;

  @override
  Widget build(BuildContext context) {
    final status = expiryStatusFor(expiry: document.expiryDate, today: today);
    return Container(
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
            child: const Icon(
              Icons.description_outlined,
              color: VeyraColors.accent,
              size: 22,
            ),
          ),
          const SizedBox(width: 14),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  document.title,
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                  style: soraDisplay(size: 15),
                ),
                const SizedBox(height: 4),
                Text(_meta(document), style: plexMono(size: 12)),
              ],
            ),
          ),
          const SizedBox(width: 10),
          _ExpiryPill(status: status),
        ],
      ),
    );
  }

  String _meta(Document d) {
    if (d.expiryDate == null) return 'No expiry · ${d.docType.label}';
    return 'Expires ${_formatDate(d.expiryDate!)}';
  }
}

class _ExpiryPill extends StatelessWidget {
  const _ExpiryPill({required this.status});
  final ExpiryStatus status;

  @override
  Widget build(BuildContext context) {
    final color = _color(status);
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 5),
      decoration: BoxDecoration(
        color: color.withValues(alpha: 0.14),
        borderRadius: BorderRadius.circular(999),
        border: Border.all(color: color.withValues(alpha: 0.3)),
      ),
      child: Text(
        status.label,
        style: TextStyle(
          color: color,
          fontSize: 11,
          fontWeight: FontWeight.w600,
        ),
      ),
    );
  }

  Color _color(ExpiryStatus status) => switch (status) {
    ExpiryStatus.expired => VeyraColors.danger,
    ExpiryStatus.expiringSoon => VeyraColors.accent,
    ExpiryStatus.valid => VeyraColors.info,
    ExpiryStatus.onFile => VeyraColors.textMuted,
  };
}

class _EmptyDocs extends StatelessWidget {
  const _EmptyDocs();

  @override
  Widget build(BuildContext context) => Center(
    child: Padding(
      padding: const EdgeInsets.symmetric(horizontal: 40),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          const Icon(
            Icons.folder_open_outlined,
            color: VeyraColors.textMuted,
            size: 40,
          ),
          const SizedBox(height: 16),
          Text('No documents yet', style: soraDisplay(size: 18)),
          const SizedBox(height: 10),
          const Text(
            'Add the STNK, BPKB, insurance, or any document you want to '
            'keep with this vehicle.',
            textAlign: TextAlign.center,
            style: TextStyle(
              color: VeyraColors.textMuted,
              fontSize: 14,
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
          Text("Can't load documents", style: soraDisplay(size: 18)),
          const SizedBox(height: 10),
          Text(
            message,
            textAlign: TextAlign.center,
            style: const TextStyle(
              color: VeyraColors.textMuted,
              fontSize: 14,
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

class _DocsSkeleton extends StatelessWidget {
  const _DocsSkeleton();

  @override
  Widget build(BuildContext context) => ListView(
    padding: const EdgeInsets.fromLTRB(20, 8, 20, 20),
    children: [
      for (var i = 0; i < 4; i++) ...[
        Container(
          height: 78,
          decoration: BoxDecoration(
            color: VeyraColors.surface,
            borderRadius: BorderRadius.circular(16),
            border: Border.all(color: VeyraColors.border),
          ),
        ),
        const SizedBox(height: 12),
      ],
    ],
  );
}

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

String _formatDate(DateTime d) => '${d.day} ${_months[d.month - 1]} ${d.year}';
