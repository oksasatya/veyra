import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:veyra_mobile/core/theme/app_theme.dart';
import 'package:veyra_mobile/core/widgets/veyra_mark.dart';
import 'package:veyra_mobile/features/auth/presentation/controllers/auth_controller.dart';

/// Garage home. Placeholder until Plan 2 wires the vehicle list + summary.
class HomeScreen extends ConsumerWidget {
  const HomeScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final user = ref.watch(authControllerProvider).asData?.value;
    return Scaffold(
      body: SafeArea(
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 20),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              const SizedBox(height: 12),
              Row(
                children: [
                  Expanded(child: Text('Garage', style: soraDisplay(size: 32))),
                  IconButton(
                    onPressed: () =>
                        ref.read(authControllerProvider.notifier).logout(),
                    icon: const Icon(Icons.logout, color: VeyraColors.textMuted),
                  ),
                ],
              ),
              if (user != null)
                Text(
                  'Signed in as ${user.name}',
                  style: const TextStyle(
                    color: VeyraColors.textMuted,
                    fontSize: 14,
                  ),
                ),
              const Spacer(),
              const _EmptyGarage(),
              const Spacer(),
            ],
          ),
        ),
      ),
    );
  }
}

class _EmptyGarage extends StatelessWidget {
  const _EmptyGarage();

  @override
  Widget build(BuildContext context) => Column(
        children: [
          Container(
            width: 108,
            height: 108,
            decoration: BoxDecoration(
              color: VeyraColors.surface,
              borderRadius: BorderRadius.circular(28),
              border: Border.all(color: VeyraColors.border),
            ),
            alignment: Alignment.center,
            child: const VeyraMark(size: 52),
          ),
          const SizedBox(height: 26),
          Text('Add your first vehicle', style: soraDisplay(size: 21)),
          const SizedBox(height: 10),
          const Padding(
            padding: EdgeInsets.symmetric(horizontal: 40),
            child: Text(
              'Track services, fuel, expenses, and reminders once your car or '
              'bike is in the garage.',
              textAlign: TextAlign.center,
              style: TextStyle(
                color: VeyraColors.textMuted,
                fontSize: 15,
                height: 1.5,
              ),
            ),
          ),
          const SizedBox(height: 24),
          FilledButton.icon(
            onPressed: null,
            icon: const Icon(Icons.add, size: 20),
            label: const Text('Add vehicle'),
          ),
        ],
      );
}
