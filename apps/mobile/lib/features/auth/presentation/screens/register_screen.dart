import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:veyra_mobile/core/theme/app_theme.dart';
import 'package:veyra_mobile/core/widgets/app_background.dart';
import 'package:veyra_mobile/features/auth/domain/value_objects/email.dart';
import 'package:veyra_mobile/features/auth/domain/value_objects/password.dart';
import 'package:veyra_mobile/features/auth/presentation/controllers/auth_controller.dart';

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
    if (_name.text.trim().isEmpty) {
      setState(() => _error = 'Enter your name.');
      return;
    }
    final email = Email.create(_email.text).toNullable();
    final password = Password.create(_password.text).toNullable();
    if (email == null) {
      setState(() => _error = 'Enter a valid email address.');
      return;
    }
    if (password == null) {
      setState(() => _error = 'Password must be at least 8 characters.');
      return;
    }
    setState(() => _error = null);
    final failure = await ref
        .read(authControllerProvider.notifier)
        .register(email, password, _name.text.trim());
    if (failure != null && mounted) {
      setState(() => _error = failure.message);
    }
  }

  @override
  Widget build(BuildContext context) {
    final loading = ref.watch(authControllerProvider).isLoading;
    return Scaffold(
      appBar: AppBar(backgroundColor: Colors.transparent),
      body: AppBackground(
        child: SafeArea(
          child: SingleChildScrollView(
            padding: const EdgeInsets.symmetric(horizontal: 24),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                Text('Create your account', style: soraDisplay(size: 26)),
                const SizedBox(height: 8),
                const Text(
                  'One account keeps every vehicle, log, and reminder in sync.',
                  style: TextStyle(color: VeyraColors.textMuted, fontSize: 15),
                ),
                const SizedBox(height: 24),
                const _Label('Name'),
                TextField(
                  controller: _name,
                  textCapitalization: TextCapitalization.words,
                  decoration: const InputDecoration(hintText: 'Your name'),
                ),
                const SizedBox(height: 14),
                const _Label('Email'),
                TextField(
                  controller: _email,
                  keyboardType: TextInputType.emailAddress,
                  autocorrect: false,
                  decoration: const InputDecoration(
                    hintText: 'you@example.com',
                  ),
                ),
                const SizedBox(height: 14),
                const _Label('Password'),
                TextField(
                  controller: _password,
                  obscureText: true,
                  decoration: const InputDecoration(
                    hintText: 'At least 8 characters',
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
                      : const Text('Create account'),
                ),
                const SizedBox(height: 16),
                Center(
                  child: GestureDetector(
                    onTap: () => context.pop(),
                    child: const Text(
                      'Already have an account? Log in',
                      style: TextStyle(
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
