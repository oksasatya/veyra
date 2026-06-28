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

  @override
  String get commonRetry => 'Coba lagi';

  @override
  String get commonServices => 'Servis';

  @override
  String get commonExpenses => 'Pengeluaran';

  @override
  String get garageTitle => 'Garasi';

  @override
  String get garageNavGarage => 'Garasi';

  @override
  String get garageNavReminders => 'Pengingat';

  @override
  String get garageNavDocs => 'Dokumen';

  @override
  String get garageNavSettings => 'Pengaturan';

  @override
  String get garageOverviewVehicles => 'Kendaraan';

  @override
  String get garageOverviewDueSoon => 'Segera jatuh tempo';

  @override
  String get garageOverviewSpent => 'Total biaya';

  @override
  String get garageCardFuel => 'BBM';

  @override
  String garageCardOdometer(String value) {
    return 'Odometer $value km';
  }

  @override
  String garageDueBadge(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count jatuh tempo',
      one: '1 jatuh tempo',
    );
    return '$_temp0';
  }

  @override
  String get garageAddVehicle => 'Tambah kendaraan';

  @override
  String get garageEmptyTitle => 'Tambah kendaraan pertama Anda';

  @override
  String get garageEmptyBody =>
      'Lacak servis, BBM, pengeluaran, dan pengingat setelah kendaraan masuk ke garasi.';

  @override
  String get garageErrorTitle => 'Tidak dapat menghubungi Veyra';

  @override
  String get garageErrorBody =>
      'Garasi gagal dimuat. Periksa koneksi Anda, lalu coba lagi.';

  @override
  String get garageErrorTryAgain => 'Coba lagi';

  @override
  String get garageErrorCheckSettings => 'Periksa pengaturan server';

  @override
  String get settingsLogOut => 'Keluar';

  @override
  String get vehicleDetailTabOverview => 'Ikhtisar';

  @override
  String get vehicleDetailTabFuel => 'BBM';

  @override
  String get vehicleDetailTabService => 'Servis';

  @override
  String get vehicleDetailTabExpenses => 'Pengeluaran';

  @override
  String get vehicleDetailTabDocs => 'Dokumen';

  @override
  String get vehicleDetailStatServices => 'Servis';

  @override
  String get vehicleDetailStatServiceCost => 'Biaya servis';

  @override
  String get vehicleDetailStatRefuels => 'Pengisian BBM';

  @override
  String get vehicleDetailStatFuelCost => 'Biaya BBM';

  @override
  String get vehicleDetailOdometerLabel => 'Odometer';

  @override
  String get vehicleDetailDueSoon => 'Segera jatuh tempo';

  @override
  String vehicleDetailReminders(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count pengingat',
      one: '1 pengingat',
    );
    return '$_temp0';
  }

  @override
  String get vehicleDetailAddFuel => 'Catat BBM';

  @override
  String get vehicleDetailAddService => 'Tambah servis';

  @override
  String get vehicleDetailAddExpense => 'Tambah pengeluaran';

  @override
  String get vehicleDetailAddDocument => 'Tambah dokumen';

  @override
  String get vehicleDetailActivityHint =>
      'Buka tab BBM, Servis, Pengeluaran, atau Dokumen untuk melihat catatan.';

  @override
  String get vehicleDetailErrorRetry => 'Coba lagi';

  @override
  String get vehicleAddTitle => 'Tambah kendaraan';

  @override
  String get vehicleAddFieldBrand => 'Merek';

  @override
  String get vehicleAddFieldModel => 'Model';

  @override
  String get vehicleAddFieldYear => 'Tahun';

  @override
  String get vehicleAddFieldPlate => 'Nomor pelat';

  @override
  String get vehicleAddFieldFuelType => 'Jenis bahan bakar';

  @override
  String get vehicleAddFieldOdometer => 'Odometer (km)';

  @override
  String get vehicleAddFieldColor => 'Warna (opsional)';

  @override
  String get vehicleAddSave => 'Simpan kendaraan';

  @override
  String get vehicleAddErrorBrandModel => 'Masukkan merek dan model kendaraan.';

  @override
  String get vehicleAddErrorYear => 'Masukkan tahun yang valid.';

  @override
  String get vehicleAddErrorPlate => 'Masukkan nomor pelat.';

  @override
  String get vehicleAddErrorOdometer =>
      'Masukkan pembacaan odometer yang valid.';

  @override
  String get fuelTypePetrol => 'Bensin';

  @override
  String get fuelTypeDiesel => 'Solar';

  @override
  String get fuelTypeElectric => 'Listrik';

  @override
  String get fuelTypeHybrid => 'Hibrida';
}
