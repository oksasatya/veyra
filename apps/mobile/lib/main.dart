import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:veyra_mobile/core/i18n/locale_controller.dart';
import 'package:veyra_mobile/core/router/app_router.dart';
import 'package:veyra_mobile/core/theme/app_theme.dart';
import 'package:veyra_mobile/l10n/app_localizations.dart';

void main() => runApp(const ProviderScope(child: VeyraApp()));

class VeyraApp extends ConsumerWidget {
  const VeyraApp({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final router = ref.watch(routerProvider);
    final locale = ref.watch(localeControllerProvider);
    return MaterialApp.router(
      onGenerateTitle: (context) => AppLocalizations.of(context).appTitle,
      debugShowCheckedModeBanner: false,
      theme: buildVeyraTheme(),
      routerConfig: router,
      // null → follow the device locale; a value overrides it (set in settings).
      locale: locale,
      localizationsDelegates: AppLocalizations.localizationsDelegates,
      supportedLocales: AppLocalizations.supportedLocales,
    );
  }
}
