import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:veyra_mobile/core/theme/app_theme.dart';
import 'package:veyra_mobile/core/widgets/veyra_mark.dart';
import 'package:veyra_mobile/features/auth/domain/value_objects/email.dart';
import 'package:veyra_mobile/features/auth/domain/value_objects/password.dart';
import 'package:veyra_mobile/features/auth/presentation/controllers/auth_controller.dart';

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
    final emailR = Email.create(_email.text);
    final passwordR = Password.create(_password.text);
    final email = emailR.toNullable();
    final password = passwordR.toNullable();
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
        .login(email, password);
    if (failure != null && mounted) {
      setState(() => _error = failure.message);
    }
  }

  @override
  Widget build(BuildContext context) {
    final loading = ref.watch(authControllerProvider).isLoading;
    return Scaffold(
      body: Stack(
        children: [
          const _TopGlow(),
          SafeArea(
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
                  Text('Welcome back', style: soraDisplay(size: 26)),
                  const SizedBox(height: 8),
                  const Text(
                    'Sign in to track your vehicles, fuel, and services.',
                    style: TextStyle(
                      color: VeyraColors.textMuted,
                      fontSize: 15,
                    ),
                  ),
                  const SizedBox(height: 28),
                  TextField(
                    controller: _email,
                    keyboardType: TextInputType.emailAddress,
                    autocorrect: false,
                    decoration: const InputDecoration(
                      hintText: 'Email',
                      prefixIcon: Icon(Icons.mail_outline, size: 20),
                    ),
                  ),
                  const SizedBox(height: 12),
                  TextField(
                    controller: _password,
                    obscureText: _obscure,
                    decoration: InputDecoration(
                      hintText: 'Password',
                      prefixIcon: const Icon(Icons.lock_outline, size: 20),
                      suffixIcon: TextButton(
                        onPressed: () => setState(() => _obscure = !_obscure),
                        child: Text(_obscure ? 'Show' : 'Hide'),
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
                        : const Text('Log in'),
                  ),
                  const Spacer(),
                  Padding(
                    padding: const EdgeInsets.only(bottom: 16),
                    child: Row(
                      mainAxisAlignment: MainAxisAlignment.center,
                      children: [
                        const Text(
                          'New to Veyra? ',
                          style: TextStyle(
                            color: VeyraColors.textMuted,
                            fontSize: 15,
                          ),
                        ),
                        GestureDetector(
                          onTap: () => context.push('/register'),
                          child: const Text(
                            'Create account',
                            style: TextStyle(
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
        ],
      ),
    );
  }
}

class _TopGlow extends StatelessWidget {
  const _TopGlow();

  @override
  Widget build(BuildContext context) => Positioned(
    top: -120,
    left: 0,
    right: 0,
    child: Center(
      child: Container(
        width: 360,
        height: 360,
        decoration: const BoxDecoration(
          shape: BoxShape.circle,
          gradient: RadialGradient(
            colors: [Color(0x38F26A21), Color(0x00F26A21)],
          ),
        ),
      ),
    ),
  );
}
