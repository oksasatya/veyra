import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:veyra_mobile/core/error/failure_l10n.dart';
import 'package:veyra_mobile/core/theme/app_theme.dart';
import 'package:veyra_mobile/core/widgets/app_background.dart';
import 'package:veyra_mobile/core/widgets/veyra_mark.dart';
import 'package:veyra_mobile/features/auth/domain/value_objects/email.dart';
import 'package:veyra_mobile/features/auth/domain/value_objects/password.dart';
import 'package:veyra_mobile/features/auth/presentation/controllers/auth_controller.dart';
import 'package:veyra_mobile/l10n/app_localizations.dart';

class LoginScreen extends ConsumerStatefulWidget {
  const LoginScreen({super.key});

  @override
  ConsumerState<LoginScreen> createState() => _LoginScreenState();
}

class _LoginScreenState extends ConsumerState<LoginScreen> {
  final _email = TextEditingController();
  final _password = TextEditingController();
  String? _error;
  bool _obscure = true;

  @override
  void dispose() {
    _email.dispose();
    _password.dispose();
    super.dispose();
  }

  Future<void> _submit() async {
    final l10n = AppLocalizations.of(context);
    final emailR = Email.create(_email.text);
    final passwordR = Password.create(_password.text);
    final email = emailR.toNullable();
    final password = passwordR.toNullable();
    if (email == null) {
      setState(() => _error = l10n.errorInvalidEmail);
      return;
    }
    if (password == null) {
      setState(() => _error = l10n.errorPasswordTooShort);
      return;
    }
    setState(() => _error = null);
    final failure = await ref
        .read(authControllerProvider.notifier)
        .login(email, password);
    if (failure != null && mounted) {
      final postL10n = AppLocalizations.of(context);
      setState(() => _error = localizedFailure(postL10n, failure));
    }
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final loading = ref.watch(authControllerProvider).isLoading;
    return Scaffold(
      body: AppBackground(
        variant: AmbientVariant.auth,
        child: SafeArea(
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 24),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                const SizedBox(height: 56),
                Row(
                  children: [
                    const VeyraMark(size: 40),
                    const SizedBox(width: 12),
                    Text('veyra', style: soraDisplay(size: 30)),
                  ],
                ),
                const SizedBox(height: 36),
                Text(l10n.authWelcomeBack, style: soraDisplay(size: 26)),
                const SizedBox(height: 8),
                Text(
                  l10n.authSubtitle,
                  style: const TextStyle(
                    color: VeyraColors.textMuted,
                    fontSize: 15,
                  ),
                ),
                const SizedBox(height: 28),
                TextField(
                  controller: _email,
                  keyboardType: TextInputType.emailAddress,
                  autocorrect: false,
                  decoration: InputDecoration(
                    hintText: l10n.authEmailHint,
                    prefixIcon: const Icon(Icons.mail_outline, size: 20),
                  ),
                ),
                const SizedBox(height: 12),
                TextField(
                  controller: _password,
                  obscureText: _obscure,
                  decoration: InputDecoration(
                    hintText: l10n.authPasswordHint,
                    prefixIcon: const Icon(Icons.lock_outline, size: 20),
                    suffixIcon: TextButton(
                      onPressed: () => setState(() => _obscure = !_obscure),
                      child: Text(_obscure ? l10n.authShow : l10n.authHide),
                    ),
                  ),
                ),
                const SizedBox(height: 10),
                Align(
                  alignment: Alignment.centerRight,
                  child: GestureDetector(
                    onTap: () {},
                    child: Text(
                      l10n.authForgotPassword,
                      style: const TextStyle(
                        color: VeyraColors.textMuted,
                        fontSize: 14,
                      ),
                    ),
                  ),
                ),
                if (_error != null) ...[
                  const SizedBox(height: 12),
                  Text(
                    _error!,
                    style: const TextStyle(
                      color: VeyraColors.danger,
                      fontSize: 13,
                    ),
                  ),
                ],
                const SizedBox(height: 24),
                FilledButton(
                  onPressed: loading ? null : _submit,
                  child: loading
                      ? const SizedBox(
                          height: 22,
                          width: 22,
                          child: CircularProgressIndicator(
                            strokeWidth: 2.4,
                            color: VeyraColors.bg,
                          ),
                        )
                      : Text(l10n.authLogIn),
                ),
                const Spacer(),
                Padding(
                  padding: const EdgeInsets.only(bottom: 16),
                  child: Row(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      Text(
                        l10n.authNewToVeyra,
                        style: const TextStyle(
                          color: VeyraColors.textMuted,
                          fontSize: 15,
                        ),
                      ),
                      GestureDetector(
                        onTap: () => context.push('/register'),
                        child: Text(
                          l10n.authCreateAccount,
                          style: const TextStyle(
                            color: VeyraColors.accent,
                            fontSize: 15,
                            fontWeight: FontWeight.w600,
                          ),
                        ),
                      ),
                    ],
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}
