import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:veyra_mobile/core/error/failure_l10n.dart';
import 'package:veyra_mobile/core/theme/app_theme.dart';
import 'package:veyra_mobile/features/document/data/repositories/document_repository_impl.dart';
import 'package:veyra_mobile/features/document/domain/repositories/document_repository.dart';
import 'package:veyra_mobile/features/document/domain/value_objects/doc_type.dart';
import 'package:veyra_mobile/l10n/app_localizations.dart';

/// Bottom-sheet form to add a document to a vehicle. Pops on success.
class AddDocumentSheet extends ConsumerStatefulWidget {
  const AddDocumentSheet({required this.vehicleId, super.key});
  final String vehicleId;

  @override
  ConsumerState<AddDocumentSheet> createState() => _AddDocumentSheetState();
}

class _AddDocumentSheetState extends ConsumerState<AddDocumentSheet> {
  final _title = TextEditingController();
  final _fileUrl = TextEditingController();
  final _notes = TextEditingController();
  DocType _docType = DocType.stnk;
  DateTime? _expiry;
  String? _error;
  bool _saving = false;

  @override
  void dispose() {
    _title.dispose();
    _fileUrl.dispose();
    _notes.dispose();
    super.dispose();
  }

  Future<void> _pickExpiry() async {
    final now = DateTime.now();
    final picked = await showDatePicker(
      context: context,
      initialDate: _expiry ?? now,
      firstDate: DateTime(now.year - 5),
      lastDate: DateTime(now.year + 20),
    );
    if (picked != null) setState(() => _expiry = picked);
  }

  Future<void> _submit() async {
    final l10n = AppLocalizations.of(context);
    if (_title.text.trim().isEmpty) {
      setState(() => _error = l10n.documentErrorEnterTitle);
      return;
    }
    setState(() {
      _error = null;
      _saving = true;
    });
    final url = _fileUrl.text.trim();
    final notes = _notes.text.trim();
    final result = await ref.read(createDocumentUseCaseProvider)(
      CreateDocumentInput(
        vehicleId: widget.vehicleId,
        docType: _docType,
        title: _title.text.trim(),
        expiryDate: _expiry,
        fileUrl: url.isEmpty ? null : url,
        notes: notes.isEmpty ? null : notes,
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
        ref.invalidate(documentListProvider(widget.vehicleId));
        Navigator.of(context).pop();
      },
    );
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final inset = MediaQuery.of(context).viewInsets.bottom;
    return Padding(
      padding: EdgeInsets.only(bottom: inset),
      child: Container(
        decoration: const BoxDecoration(
          color: VeyraColors.bg,
          borderRadius: BorderRadius.vertical(top: Radius.circular(20)),
        ),
        child: SafeArea(
          top: false,
          child: SingleChildScrollView(
            padding: const EdgeInsets.fromLTRB(20, 14, 20, 20),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                const _Grabber(),
                const SizedBox(height: 14),
                Text(l10n.documentAddTitle, style: soraDisplay(size: 20)),
                const SizedBox(height: 18),
                _Label(l10n.documentFieldType),
                _DocTypeSelector(
                  value: _docType,
                  onChanged: (t) => setState(() => _docType = t),
                ),
                const SizedBox(height: 16),
                _field(
                  l10n.documentFieldTitle,
                  _title,
                  hint: l10n.documentFieldTitleHint,
                ),
                _Label(l10n.documentFieldExpiry),
                _DateField(value: _expiry, onTap: _pickExpiry),
                const SizedBox(height: 16),
                _field(
                  l10n.documentFieldFileUrl,
                  _fileUrl,
                  hint: l10n.documentFieldFileUrlHint,
                ),
                _field(
                  l10n.documentFieldNotes,
                  _notes,
                  hint: l10n.documentFieldNotesHint,
                ),
                if (_error != null) ...[
                  const SizedBox(height: 6),
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
                      : Text(l10n.documentSave),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }

  Widget _field(
    String label,
    TextEditingController controller, {
    String? hint,
  }) => Column(
    crossAxisAlignment: CrossAxisAlignment.start,
    children: [
      _Label(label),
      TextField(
        controller: controller,
        decoration: InputDecoration(hintText: hint),
      ),
      const SizedBox(height: 16),
    ],
  );
}

class _Grabber extends StatelessWidget {
  const _Grabber();

  @override
  Widget build(BuildContext context) => Center(
    child: Container(
      width: 40,
      height: 4,
      decoration: BoxDecoration(
        color: VeyraColors.border,
        borderRadius: BorderRadius.circular(2),
      ),
    ),
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

class _DocTypeSelector extends StatelessWidget {
  const _DocTypeSelector({required this.value, required this.onChanged});
  final DocType value;
  final ValueChanged<DocType> onChanged;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    return Wrap(
      spacing: 8,
      runSpacing: 8,
      children: [
        for (final t in DocType.values)
          ChoiceChip(
            label: Text(_localizedDocType(l10n, t)),
            selected: t == value,
            onSelected: (_) => onChanged(t),
            backgroundColor: VeyraColors.surface,
            selectedColor: VeyraColors.accent,
            labelStyle: TextStyle(
              color: t == value ? VeyraColors.bg : VeyraColors.text,
              fontWeight: FontWeight.w500,
            ),
            side: const BorderSide(color: VeyraColors.border),
          ),
      ],
    );
  }
}

class _DateField extends StatelessWidget {
  const _DateField({required this.value, required this.onTap});
  final DateTime? value;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    return InkWell(
      onTap: onTap,
      borderRadius: BorderRadius.circular(14),
      child: InputDecorator(
        decoration: const InputDecoration(),
        child: Row(
          children: [
            Expanded(
              child: Text(
                value == null ? l10n.documentNoExpiry : _formatDate(value!),
                style: TextStyle(
                  color: value == null
                      ? const Color(0xFF5A6472)
                      : VeyraColors.text,
                  fontSize: 16,
                ),
              ),
            ),
            const Icon(
              Icons.calendar_today_outlined,
              color: VeyraColors.textMuted,
              size: 18,
            ),
          ],
        ),
      ),
    );
  }
}

String _localizedDocType(AppLocalizations l10n, DocType type) => switch (type) {
  DocType.stnk => l10n.docTypeStnk,
  DocType.bpkb => l10n.docTypeBpkb,
  DocType.insurance => l10n.docTypeInsurance,
  DocType.other => l10n.docTypeOther,
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

String _formatDate(DateTime d) => '${d.day} ${_months[d.month - 1]} ${d.year}';
