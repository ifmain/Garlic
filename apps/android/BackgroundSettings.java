package org.garlic.client;

import android.app.Activity;
import android.app.ActivityManager;
import android.app.AlertDialog;
import android.app.NotificationManager;
import android.content.ActivityNotFoundException;
import android.content.Context;
import android.content.Intent;
import android.net.Uri;
import android.net.VpnService;
import android.system.Os;
import android.system.ErrnoException;
import android.os.PowerManager;
import android.provider.Settings;
import android.widget.Toast;

public final class BackgroundSettings {
    private static final int EXPORT_LOG = 4701;

    private static java.io.File logs(Context a) {
        return new java.io.File(a.getFilesDir(), "garlic/logs");
    }

    public static void recordLaunch(Activity a) {
        record(a, "app_start android_sdk=" + android.os.Build.VERSION.SDK_INT);
    }

    public static void serviceError(Context context, Throwable cause) {
        record(context, "vpn_start_error " + cause.getClass().getSimpleName());
    }

    private static void record(Context a, String event) {
        java.io.File dir = logs(a);
        if (!dir.isDirectory() && !dir.mkdirs()) return;
        java.time.Instant now = java.time.Instant.now();
        java.io.File log = new java.io.File(dir, now.toString().substring(0, 10) + ".log");
        if (log.length() >= 8L * 1024 * 1024) return;
        try (java.io.FileOutputStream out = new java.io.FileOutputStream(log, true)) {
            out.write((now + " " + event + "\n")
                .getBytes(java.nio.charset.StandardCharsets.UTF_8));
        } catch (java.io.IOException ignored) {}
    }

    public static void exportLogs(Activity a) {
        Intent intent = new Intent(Intent.ACTION_CREATE_DOCUMENT).addCategory(Intent.CATEGORY_OPENABLE)
            .setType("text/plain").putExtra(Intent.EXTRA_TITLE, "Garlic-" + java.time.LocalDate.now() + ".log");
        try { a.startActivityForResult(intent, EXPORT_LOG); }
        catch (ActivityNotFoundException e) { Toast.makeText(a, "Нет приложения для сохранения файла", Toast.LENGTH_LONG).show(); }
    }

    public static void addExportButton(Activity a, android.widget.LinearLayout parent) {
        android.widget.Button button = new android.widget.Button(a);
        button.setText("Экспорт журнала");
        button.setOnClickListener(v -> exportLogs(a));
        parent.addView(button);
    }

    public static void exportResult(Activity a, int request, int result, Intent data) {
        if (request != EXPORT_LOG || result != Activity.RESULT_OK || data == null || data.getData() == null) return;
        new Thread(() -> {
            String message;
            try {
                try (java.io.OutputStream out = a.getContentResolver().openOutputStream(data.getData(), "wt")) {
                    if (out == null) throw new java.io.IOException();
                    exportDays(logs(a), out);
                }
                message = "Журнал сохранён";
            } catch (Exception e) { message = "Не удалось сохранить журнал"; }
            final String status = message;
            a.runOnUiThread(() -> Toast.makeText(a, status, Toast.LENGTH_LONG).show());
        }, "garlic-log-export").start();
    }

    static void exportDays(java.io.File directory, java.io.OutputStream out) throws java.io.IOException {
        java.io.File[] files = directory.listFiles((dir, name) -> name.matches("[0-9]{4}-[0-9]{2}-[0-9]{2}\\.log"));
        if (files == null || files.length == 0) throw new java.io.IOException("No daily logs");
        java.util.Arrays.sort(files);
        for (int i = Math.max(0, files.length - 7); i < files.length; i++)
            java.nio.file.Files.copy(files[i].toPath(), out);
    }
    private static boolean consideredStartup;

    public static boolean shouldStart(Activity activity) {
        if (consideredStartup) return false;
        consideredStartup = true;
        if (activity.getSystemService(ActivityManager.class).isBackgroundRestricted()) {
            new AlertDialog.Builder(activity).setTitle("Android ограничивает работу Garlic в фоне")
                .setMessage("Чтобы VPN мог продолжать соединение после сворачивания приложения, проверьте настройки фоновой работы и батареи для Garlic Alpha.")
                .setNegativeButton("Позже", null)
                .setPositiveButton("Настройки", (d, w) -> openSettings(activity, appSettings(activity))).show();
        }
        return activity.getSharedPreferences("garlic_prefs", 0).getBoolean("alpha_auto_start", false)
            && VpnService.prepare(activity) == null;
    }
    private static String protocols(Context context) {
        return context.getSharedPreferences("garlic_prefs", 0).getString("alpha_protocols", "2,3,4");
    }

    public static void applyProtocols(Context context) {
        try {
            Os.setenv("GARLIC_ALPHA_PROTOCOLS", protocols(context), true);
            Os.setenv("GARLIC_ALPHA_SLOT", context.getSharedPreferences("garlic_prefs", 0)
                .getBoolean("alpha_slot", true) ? "1" : "0", true);
        } catch (ErrnoException e) {
            throw new IllegalStateException("Unable to apply protocol preferences", e);
        }
    }

    public static void chooseProtocols(Activity activity) {
        String current = protocols(activity);
        boolean[] selected = {current.contains("2"), current.contains("3"), current.contains("4"),
            activity.getSharedPreferences("garlic_prefs", 0).getBoolean("alpha_slot", true)};
        AlertDialog dialog = new AlertDialog.Builder(activity).setTitle("Протоколы")
            .setMultiChoiceItems(new String[]{"GEX2", "GEX3", "GEX4 (экспериментальный)", "Обновлять профиль протокола"}, selected,
                (d, i, on) -> selected[i] = on)
            .setNegativeButton(android.R.string.cancel, null)
            .setPositiveButton(android.R.string.ok, null).create();
        dialog.show();
        dialog.getButton(AlertDialog.BUTTON_POSITIVE).setOnClickListener(v -> {
            java.util.ArrayList<String> versions = new java.util.ArrayList<>();
            for (int i = 0; i < 3; i++) {
                if (selected[i]) versions.add(String.valueOf(i + 2));
            }
            if (versions.isEmpty()) {
                Toast.makeText(activity, "Выберите хотя бы один протокол", Toast.LENGTH_SHORT).show();
                return;
            }
            activity.getSharedPreferences("garlic_prefs", 0).edit()
                .putBoolean("alpha_slot", selected[3])
                .putString("alpha_protocols", android.text.TextUtils.join(",", versions)).apply();
            applyProtocols(activity);
            dialog.dismiss();
        });
    }
    public static void show(Activity a) {
        ActivityManager manager = (ActivityManager)a.getSystemService(Context.ACTIVITY_SERVICE);
        String background = manager.isBackgroundRestricted()
            ? "Android ограничивает работу в фоне · изменить"
            : "Системное ограничение фоновой работы выключено · настройки";
        PowerManager power = (PowerManager)a.getSystemService(Context.POWER_SERVICE);
        NotificationManager notifications = (NotificationManager)a.getSystemService(Context.NOTIFICATION_SERVICE);
        String battery = power.isIgnoringBatteryOptimizations(a.getPackageName())
            ? "Без оптимизации батареи" : "Системная оптимизация батареи";
        String notice = notifications.areNotificationsEnabled()
            ? "Уведомления включены" : "Уведомления выключены";
        new AlertDialog.Builder(a).setTitle("Работа в фоне")
            .setItems(new String[]{background, battery, notice,
                "Подключать при открытии: " + (a.getSharedPreferences("garlic_prefs", 0).getBoolean("alpha_auto_start", false) ? "да" : "нет")}, (dialog, which) -> {
                Intent intent;
                switch (which) {
                    case 3:
                        android.content.SharedPreferences prefs = a.getSharedPreferences("garlic_prefs", 0);
                        prefs.edit().putBoolean("alpha_auto_start", !prefs.getBoolean("alpha_auto_start", false)).apply();
                        show(a);
                        return;
                    case 0:
                        intent = appSettings(a);
                        break;
                    case 1: intent = new Intent(Settings.ACTION_IGNORE_BATTERY_OPTIMIZATION_SETTINGS); break;
                    default: intent = new Intent(Settings.ACTION_APP_NOTIFICATION_SETTINGS)
                                 .putExtra(Settings.EXTRA_APP_PACKAGE, a.getPackageName());
                }
                openSettings(a, intent);
            }).show();
    }

    private static Intent appSettings(Activity a) {
        return new Intent(Settings.ACTION_APPLICATION_DETAILS_SETTINGS,
            Uri.fromParts("package", a.getPackageName(), null));
    }

    private static void openSettings(Activity a, Intent intent) {
        try { a.startActivity(intent); }
        catch (ActivityNotFoundException e) {
            Toast.makeText(a, "Откройте системные настройки приложения", Toast.LENGTH_LONG).show();
        }
    }
}
