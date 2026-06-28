import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:veyra_mobile/core/error/failure_l10n.dart';
import 'package:veyra_mobile/core/theme/app_theme.dart';
import 'package:veyra_mobile/core/widgets/app_background.dart';
import 'package:veyra_mobile/features/auth/domain/value_objects/email.dart';
import 'package:veyra_mobile/features/auth/domain/value_objects/password.dart';
import 'package:veyra_mobile/features/auth/presentation/controllers/auth_controller.dart';
import 'package:veyra_mobile/l10n/app_localizations.dart';

class RegisterScreen extends ConsumerStatefulWidget {
  const RegisterScreen({super.key});

  @override
  ConsumerState<RegisterScreen> createState() => _RegisterScreenState();
}

class _RegisterScreenState extends ConsumerState<RegisterScreen> {
  final _name = TextEditingController();
  final _email = TextEditingController();
  final _password = TextEditingController();
  String? _error;

  @override
  void dispose() {
    _name.dispose();
    _email.dispose();
    _password.dispose();
    super.dispose();
  }

  Future<void> _submit() async {
    final l10n = AppLocalizations.of(context);
    if (_name.text.trim().isEmpty) {
      setState(() => _error = l10n.authEnterName);
      return;
    }
    final email = Email.create(_email.text).toNullable();
    final password = Password.create(_password.text).toNullable();
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
        .register(email, password, _name.text.trim());
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
      appBar: AppBar(backgroundColor: Colors.transparent),
      body: AppBackground(
        variant: AmbientVariant.auth,
        child: SafeArea(
          child: SingleChildScrollView(
            padding: const EdgeInsets.symmetric(horizontal: 24),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                Text(l10n.authCreateTitle, style: soraDisplay(size: 26)),
                const SizedBox(height: 8),
                Text(
                  l10n.authCreateSubtitle,
                  style: const TextStyle(
                    color: VeyraColors.textMuted,
                    fontSize: 15,
                  ),
                ),
                const SizedBox(height: 24),
                _Label(l10n.authNameLabel),
                TextField(
                  controller: _name,
                  textCapitalization: TextCapitalization.words,
                  decoration: InputDecoration(
                    hintText: l10n.authNameHint,
                    prefixIcon: const Icon(Icons.person_outline, size: 20),
                  ),
                ),
                const SizedBox(height: 14),
                _Label(l10n.authEmailLabel),
                TextField(
                  controller: _email,
                  keyboardType: TextInputType.emailAddress,
                  autocorrect: false,
                  decoration: const InputDecoration(
                    hintText: 'you@example.com',
                    prefixIcon: Icon(Icons.mail_outline, size: 20),
                  ),
                ),
                const SizedBox(height: 14),
                _Label(l10n.authPasswordLabel),
                TextField(
                  controller: _password,
                  obscureText: true,
                  decoration: InputDecoration(
                    hintText: l10n.authPasswordHint,
                    prefixIcon: const Icon(Icons.lock_outline, size: 20),
                  ),
                ),
                const SizedBox(height: 8),
                Text(
                  l10n.authPasswordHelp,
                  style: const TextStyle(
                    color: VeyraColors.textMuted,
                    fontSize: 12,
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
                      : Text(l10n.authCreateAccount),
                ),
                const SizedBox(height: 16),
                Center(
                  child: GestureDetector(
                    onTap: () => context.pop(),
                    child: Text(
                      l10n.authAlreadyHaveAccount,
                      style: const TextStyle(
                        color: VeyraColors.textMuted,
                        fontSize: 15,
                      ),
                    ),
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
