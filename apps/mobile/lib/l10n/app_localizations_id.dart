// ignore: unused_import
import 'package:intl/intl.dart' as intl;
import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Indonesian (`id`).
class AppLocalizationsId extends AppLocalizations {
  AppLocalizationsId([String locale = 'id']) : super(locale);

  @override
  String get appTitle => 'Veyra';

  @override
  String get settingsTitle => 'Pengaturan';

  @override
  String get settingsLanguage => 'Bahasa';

  @override
  String get languageSystem => 'Ikuti sistem';

  @override
  String get languageEnglish => 'Inggris';

  @override
  String get languageIndonesian => 'Indonesia';

  @override
  String get errorNetwork => 'Tidak ada koneksi. Periksa jaringan Anda.';

  @override
  String get errorServer => 'Terjadi kesalahan di server.';

  @override
  String get errorUnauthorized =>
      'Sesi Anda telah berakhir. Silakan masuk kembali.';

  @override
  String get errorForbidden => 'Anda tidak memiliki akses ke ini.';

  @override
  String get errorNotFound => 'Tidak ditemukan.';

  @override
  String get errorConflict => 'Data tersebut sudah ada.';

  @override
  String get errorValidation => 'Periksa kembali masukan Anda.';

  @override
  String get errorServiceUnavailable =>
      'Layanan sedang tidak tersedia. Coba lagi.';

  @override
  String get errorInvalidEmail => 'Masukkan alamat email yang valid.';

  @override
  String get errorPasswordTooShort => 'Kata sandi minimal 8 karakter.';

  @override
  String get errorEmailAlreadyExists => 'Email tersebut sudah terdaftar.';

  @override
  String get errorInvalidLanguage => 'Bahasa tidak didukung.';

  @override
  String get errorInvalidPlateNumber => 'Masukkan nomor pelat yang valid.';

  @override
  String get errorOdometerDecreased =>
      'Odometer tidak boleh lebih kecil dari nilai saat ini.';

  @override
  String get errorInvalidFuelType => 'Pilih jenis bahan bakar yang valid.';

  @override
  String get errorInvalidReminderType => 'Pilih jenis pengingat yang valid.';

  @override
  String get errorMissingDueDate =>
      'Tanggal jatuh tempo wajib diisi untuk pengingat ini.';

  @override
  String get errorMissingDueOdometer =>
      'Odometer jatuh tempo wajib diisi untuk pengingat ini.';

  @override
  String get errorInvalidCategory => 'Pilih kategori yang valid.';

  @override
  String get errorInvalidDocType => 'Pilih jenis dokumen yang valid.';

  @override
  String get authWelcomeBack => 'Selamat datang kembali';

  @override
  String get authSubtitle =>
      'Masuk untuk melacak kendaraan, bahan bakar, dan servis Anda.';

  @override
  String get authEmailHint => 'Email';

  @override
  String get authEmailLabel => 'Email';

  @override
  String get authPasswordHint => 'Kata sandi';

  @override
  String get authPasswordLabel => 'Kata sandi';

  @override
  String get authShow => 'Tampilkan';

  @override
  String get authHide => 'Sembunyikan';

  @override
  String get authForgotPassword => 'Lupa kata sandi?';

  @override
  String get authLogIn => 'Masuk';

  @override
  String get authNewToVeyra => 'Baru di Veyra? ';

  @override
  String get authCreateAccount => 'Buat akun';

  @override
  String get authCreateTitle => 'Buat akun Anda';

  @override
  String get authCreateSubtitle =>
      'Satu akun menyimpan semua kendaraan, catatan, dan pengingat Anda.';

  @override
  String get authNameLabel => 'Nama';

  @override
  String get authNameHint => 'Nama Anda';

  @override
  String get authPasswordHelp => 'Gunakan 8 karakter atau lebih.';

  @override
  String get authAlreadyHaveAccount => 'Sudah punya akun? Masuk';

  @override
  String get authEnterName => 'Masukkan nama Anda.';
}
