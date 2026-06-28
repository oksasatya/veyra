import 'package:flutter/material.dart';
import 'package:google_fonts/google_fonts.dart';

/// Veyra brand tokens — Carbon + Signal Amber, dark-first.
abstract class VeyraColors {
  static const bg = Color(0xFF0D1119);
  static const surface = Color(0xFF151A23);
  static const surface2 = Color(0xFF1B212C);
  static const border = Color(0xFF232B36);
  static const text = Color(0xFFE6EAF0);
  static const textMuted = Color(0xFF9BA6B5);
  static const accent = Color(0xFFF26A21);
  static const accentHover = Color(0xFFFF8338);
  static const info = Color(0xFF34D1C4);
  static const danger = Color(0xFFF2555A);
  static const ok = Color(0xFF4FD08A);
  static const hintText = Color(0xFF5A6472);

  // Expense-breakdown chart roles (design `--c3` / `--c4`).
  static const chartViolet = Color(0xFF7C83FF);
  static const chartSlate = Color(0xFF586072);
}

/// Sora display style for screen titles and the wordmark.
TextStyle soraDisplay({
  double size = 24,
  FontWeight weight = FontWeight.w600,
  Color color = VeyraColors.text,
}) => GoogleFonts.sora(
  fontSize: size,
  fontWeight: weight,
  color: color,
  letterSpacing: -0.4,
);

/// IBM Plex Mono style for data (odometer, money, plate, tokens).
TextStyle plexMono({
  double size = 14,
  FontWeight weight = FontWeight.w500,
  Color color = VeyraColors.textMuted,
}) => GoogleFonts.ibmPlexMono(fontSize: size, fontWeight: weight, color: color);

ThemeData buildVeyraTheme() {
  final scheme =
      ColorScheme.fromSeed(
        seedColor: VeyraColors.accent,
        brightness: Brightness.dark,
        surface: VeyraColors.surface,
      ).copyWith(
        primary: VeyraColors.accent,
        onPrimary: VeyraColors.bg,
        error: VeyraColors.danger,
        surface: VeyraColors.surface,
      );

  final textTheme = GoogleFonts.ibmPlexSansTextTheme(
    ThemeData(brightness: Brightness.dark).textTheme,
  ).apply(bodyColor: VeyraColors.text, displayColor: VeyraColors.text);

  return ThemeData(
    useMaterial3: true,
    brightness: Brightness.dark,
    colorScheme: scheme,
    scaffoldBackgroundColor: VeyraColors.bg,
    textTheme: textTheme,
    inputDecorationTheme: InputDecorationTheme(
      filled: true,
      fillColor: VeyraColors.surface,
      hintStyle: const TextStyle(color: Color(0xFF5A6472)),
      contentPadding: const EdgeInsets.symmetric(horizontal: 16, vertical: 16),
      border: _inputBorder(VeyraColors.border),
      enabledBorder: _inputBorder(VeyraColors.border),
      focusedBorder: _inputBorder(VeyraColors.accent, width: 1.6),
      errorBorder: _inputBorder(VeyraColors.danger),
      focusedErrorBorder: _inputBorder(VeyraColors.danger, width: 1.6),
    ),
    filledButtonTheme: FilledButtonThemeData(
      style: FilledButton.styleFrom(
        backgroundColor: VeyraColors.accent,
        foregroundColor: VeyraColors.bg,
        minimumSize: const Size.fromHeight(54),
        textStyle: GoogleFonts.ibmPlexSans(
          fontSize: 16,
          fontWeight: FontWeight.w600,
        ),
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(14),
        ),
      ),
    ),
  );
}

OutlineInputBorder _inputBorder(Color color, {double width = 1}) =>
    OutlineInputBorder(
      borderRadius: BorderRadius.circular(14),
      borderSide: BorderSide(color: color, width: width),
    );
