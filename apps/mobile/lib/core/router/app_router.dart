import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:veyra_mobile/features/auth/presentation/controllers/auth_controller.dart';
import 'package:veyra_mobile/features/auth/presentation/screens/login_screen.dart';
import 'package:veyra_mobile/features/auth/presentation/screens/register_screen.dart';
import 'package:veyra_mobile/features/auth/presentation/screens/splash_screen.dart';
import 'package:veyra_mobile/features/vehicle/domain/entities/vehicle.dart';
import 'package:veyra_mobile/features/vehicle/presentation/screens/add_vehicle_screen.dart';
import 'package:veyra_mobile/features/vehicle/presentation/screens/garage_screen.dart';
import 'package:veyra_mobile/features/vehicle/presentation/screens/vehicle_detail_screen.dart';

final routerProvider = Provider<GoRouter>((ref) {
  // Bridge Riverpod auth-state changes into a Listenable go_router can refresh on.
  final refresh = ValueNotifier<int>(0);
  ref
    ..onDispose(refresh.dispose)
    ..listen(authControllerProvider, (_, _) => refresh.value++);

  return GoRouter(
    initialLocation: '/splash',
    refreshListenable: refresh,
    redirect: (context, state) {
      final auth = ref.read(authControllerProvider);
      final loc = state.matchedLocation;
      // While the saved session is being restored, hold on the splash.
      if (auth.isLoading) return loc == '/splash' ? null : '/splash';
      final loggedIn = auth.asData?.value != null;
      final atAuth = loc == '/login' || loc == '/register';
      if (loc == '/splash') return loggedIn ? '/' : '/login';
      if (!loggedIn && !atAuth) return '/login';
      if (loggedIn && atAuth) return '/';
      return null;
    },
    routes: [
      GoRoute(path: '/splash', builder: (_, _) => const SplashScreen()),
      GoRoute(path: '/', builder: (_, _) => const GarageScreen()),
      GoRoute(path: '/login', builder: (_, _) => const LoginScreen()),
      GoRoute(path: '/register', builder: (_, _) => const RegisterScreen()),
      GoRoute(
        path: '/vehicles/new',
        builder: (_, _) => const AddVehicleScreen(),
      ),
      GoRoute(
        path: '/vehicles/:id',
        builder: (context, state) {
          final extra = state.extra;
          return extra is Vehicle
              ? VehicleDetailScreen(vehicle: extra)
              : const GarageScreen();
        },
      ),
    ],
  );
});
