import 'dart:async' show unawaited;

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:veyra_mobile/core/error/failure_l10n.dart';
import 'package:veyra_mobile/core/i18n/locale_controller.dart';
import 'package:veyra_mobile/core/theme/app_theme.dart';
import 'package:veyra_mobile/features/auth/data/repositories/auth_repository_impl.dart';
import 'package:veyra_mobile/features/auth/presentation/controllers/auth_controller.dart';
import 'package:veyra_mobile/l10n/app_localizations.dart';

class SettingsScreen extends ConsumerWidget {
  const SettingsScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final user = ref.watch(authControllerProvider).asData?.value;
    final currentLocale = ref.watch(localeControllerProvider);

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        // ── Header ────────────────────────────────────────────────────────────
        Padding(
          padding: const EdgeInsets.fromLTRB(20, 16, 20, 4),
          child: Text(l10n.settingsTitle, style: soraDisplay(size: 32)),
        ),
        if (user != null)
          Padding(
            padding: const EdgeInsets.fromLTRB(20, 2, 20, 0),
            child: Text(
              '${user.name} · ${user.email}',
              style: const TextStyle(
                color: VeyraColors.textMuted,
                fontSize: 14,
              ),
            ),
          ),
        const SizedBox(height: 24),

        // ── Language section ──────────────────────────────────────────────────
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 20),
          child: Text(
            l10n.settingsLanguage,
            style: const TextStyle(
              color: VeyraColors.textMuted,
              fontSize: 12,
              fontWeight: FontWeight.w500,
              letterSpacing: 0.6,
            ),
          ),
        ),
        const SizedBox(height: 8),
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 20),
          child: Container(
            decoration: BoxDecoration(
              color: VeyraColors.surface,
              borderRadius: BorderRadius.circular(14),
              border: Border.all(color: VeyraColors.border),
            ),
            child: Column(
              children: [
                _LanguageRow(
                  label: l10n.languageSystem,
                  selected: currentLocale == null,
                  onTap: () => _onLanguageTap(context, ref, null),
                ),
                const Divider(
                  height: 1,
                  thickness: 1,
                  color: VeyraColors.border,
                  indent: 20,
                  endIndent: 0,
                ),
                _LanguageRow(
                  label: l10n.languageEnglish,
                  selected:
                      currentLocale?.languageCode == 'en',
                  onTap: () =>
                      _onLanguageTap(context, ref, const Locale('en')),
                ),
                const Divider(
                  height: 1,
                  thickness: 1,
                  color: VeyraColors.border,
                  indent: 20,
                  endIndent: 0,
                ),
                _LanguageRow(
                  label: l10n.languageIndonesian,
                  selected:
                      currentLocale?.languageCode == 'id',
                  onTap: () =>
                      _onLanguageTap(context, ref, const Locale('id')),
                ),
              ],
            ),
          ),
        ),

        const Spacer(),

        // ── Log out ───────────────────────────────────────────────────────────
        Padding(
          padding: const EdgeInsets.fromLTRB(20, 0, 20, 32),
          child: FilledButton.tonalIcon(
            onPressed: () =>
                ref.read(authControllerProvider.notifier).logout(),
            icon: const Icon(Icons.logout),
            label: Text(l10n.settingsLogOut),
          ),
        ),
      ],
    );
  }

  /// Applies the locale locally (immediate) and syncs to backend fire-and-forget.
  void _onLanguageTap(
    BuildContext context,
    WidgetRef ref,
    Locale? locale,
  ) {
    // 1. Apply locally immediately — never blocks the toggle.
    unawaited(
      ref.read(localeControllerProvider.notifier).setLocale(locale),
    );

    // 2. Sync to backend if authenticated (fire-and-forget).
    final isAuthenticated =
        ref.read(authControllerProvider).asData?.value != null;
    if (!isAuthenticated) return;

    // Resolve the language code: explicit choice → its code;
    // System default → device locale clamped to supported set, fallback 'en'.
    final String code;
    if (locale != null) {
      code = locale.languageCode;
    } else {
      final deviceCode = WidgetsBinding
          .instance.platformDispatcher.locale.languageCode;
      code = supportedLanguageCodes.contains(deviceCode) ? deviceCode : 'en';
    }

    final scaffoldMessenger = ScaffoldMessenger.of(context);
    final l10n = AppLocalizations.of(context);

    unawaited(
      ref.read(updatePreferencesUseCaseProvider)(code).then((result) {
        result.fold(
          (failure) {
            scaffoldMessenger.showSnackBar(
              SnackBar(
                content: Text(localizedFailure(l10n, failure)),
                backgroundColor: VeyraColors.surface2,
                behavior: SnackBarBehavior.floating,
              ),
            );
          },
          (_) {},
        );
      }),
    );
  }
}

// ── Language row ─────────────────────────────────────────────────────────────

class _LanguageRow extends StatelessWidget {
  const _LanguageRow({
    required this.label,
    required this.selected,
    required this.onTap,
  });

  final String label;
  final bool selected;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return Semantics(
      label: label,
      selected: selected,
      button: true,
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(14),
        child: SizedBox(
          height: 52,
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 20),
            child: Row(
              children: [
                Expanded(
                  child: Text(
                    label,
                    style: TextStyle(
                      color: selected
                          ? VeyraColors.accent
                          : VeyraColors.text,
                      fontSize: 15,
                      fontWeight: selected
                          ? FontWeight.w600
                          : FontWeight.w400,
                    ),
                  ),
                ),
                if (selected)
                  const Icon(
                    Icons.check_rounded,
                    color: VeyraColors.accent,
                    size: 20,
                  ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}
