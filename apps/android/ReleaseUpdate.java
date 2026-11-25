package org.garlic.client;

import android.app.Activity;
import android.app.AlertDialog;
import android.content.Intent;
import android.content.pm.PackageInfo;
import android.content.pm.PackageManager;
import android.net.Uri;
import android.provider.Settings;
import android.widget.Toast;
import java.io.*;
import java.net.URL;
import java.util.Arrays;
import javax.net.ssl.HttpsURLConnection;
import org.json.JSONArray;
import org.json.JSONObject;

public final class ReleaseUpdate {
    private static final int PERMISSION = 4782;
    private static boolean checking, manualRequested, downloading;
    static File pending(android.content.Context c) { return new File(c.getCacheDir(), "garlic-release.apk"); }
    static void fetch(String address, OutputStream out, int limit) throws IOException {
        HttpsURLConnection c = (HttpsURLConnection)new URL(address).openConnection();
        c.setRequestProperty("User-Agent", "Garlic-Alpha");
        c.setRequestProperty("Accept-Encoding", "identity");
        c.setConnectTimeout(10000); c.setReadTimeout(20000);
        try {
            if(c.getResponseCode()!=200 || c.getContentLengthLong()>limit) throw new IOException("Download failed");
            try(InputStream in=c.getInputStream()) {
                byte[] buffer=new byte[32768]; int size=0,n;
                while((n=in.read(buffer))!=-1) {
                    size+=n; if(size>limit) throw new IOException("Download too large");
                    out.write(buffer,0,n);
                }
                long expected=c.getContentLengthLong();
                if(expected>=0 && size!=expected) throw new EOFException("Incomplete download");
            }
        } finally { c.disconnect(); }
    }
    private static boolean newer(String a, String b) {
        if(!a.matches("[0-9]{1,6}(\\.[0-9]{1,6}){2,3}") || !b.matches("[0-9]{1,6}(\\.[0-9]{1,6}){2,3}")) return false;
        String[] x=a.split("\\."), y=b.split("\\.");
        for(int i=0;i<4;i++) {
            int d=(i<x.length?Integer.parseInt(x[i]):0)-(i<y.length?Integer.parseInt(y[i]):0);
            if(d!=0) return d>0;
        }
        return false;
    }
    static boolean supportWarning(String notes, String installed, String offered) {
        if (notes == null) return false;
        String line = notes.trim().split("\n",2)[0].trim();
        java.util.regex.Matcher marker = java.util.regex.Pattern.compile(
            "<!-- garlic-support-floor: ([0-9]{1,6}(?:\\.[0-9]{1,6}){2,3}) -->").matcher(line);
        return marker.matches() && newer(marker.group(1), installed) && !newer(marker.group(1), offered);
    }
    private static boolean previews(Activity a) {
        return a.getSharedPreferences("garlic_prefs",0).getBoolean("release_previews",false);
    }
    public static String[] withChannel(String[] items) {
        String[] result=Arrays.copyOf(items,items.length+1);
        result[items.length]="Источник обновлений";
        return result;
    }
    public static void channel(Activity a) {
        new AlertDialog.Builder(a).setTitle("Источник обновлений")
            .setSingleChoiceItems(new String[]{"Стабильные релизы","Релизы и пререлизы"},previews(a)?1:0,(dialog,which)->{
                a.getSharedPreferences("garlic_prefs",0).edit().putBoolean("release_previews",which==1).apply();
                dialog.dismiss();
            }).show();
    }
    static JSONObject select(JSONArray items, String installed, boolean previews) throws Exception {
        JSONObject best=null;
        for(int i=0;i<items.length();i++) {
            JSONObject item=items.getJSONObject(i);
            String tag=item.optString("tag_name");
            if(item.getBoolean("draft") || (item.getBoolean("prerelease") && !previews)
                    || !tag.startsWith("alpha-") || !newer(tag.substring(6),installed)) continue;
            if(best==null || item.getString("published_at").compareTo(best.getString("published_at"))>0 ||
                    (item.getString("published_at").equals(best.getString("published_at")) && newer(tag.substring(6),best.getString("tag_name").substring(6)))) best=item;
        }
        return best;
    }
    public static void addButton(Activity a, android.widget.LinearLayout parent) {
        android.widget.Button button = ((SecondaryButtons)a).secondaryButton("Обновить с GitHub");
        button.setOnClickListener(v -> check(a, true));
        parent.addView(button);
        android.widget.Button plugins = ((SecondaryButtons)a).secondaryButton("Плагины");
        plugins.setOnClickListener(v -> PluginFiles.show(a));
        parent.addView(plugins);
    }
    public static void check(Activity a) { check(a, false); }
    private static void check(Activity a, boolean manual) {
        if (downloading) {
            if (manual) Toast.makeText(a,"Обновление загружается",Toast.LENGTH_SHORT).show();
            return;
        }
        manualRequested |= manual;
        if (manual) Toast.makeText(a,"Проверяем GitHub…",Toast.LENGTH_SHORT).show();
        if (checking) return;
        checking=true;
        new Thread(() -> {
            try {
                String installed=a.getPackageManager().getPackageInfo(a.getPackageName(),0).versionName;
                boolean allowPreview=previews(a);
                JSONObject selected=null;
                for(int page=1; ; page++) {
                    ByteArrayOutputStream out=new ByteArrayOutputStream();
                    fetch("https://api.github.com/repos/ifmain/Garlic/releases?per_page=30&page="+page,out,4*1024*1024);
                    JSONArray items=new JSONArray(out.toString("UTF-8"));
                    int count=items.length();
                    if(selected!=null) items.put(selected);
                    selected=select(items,installed,allowPreview);
                    if(count<30) break;
                }
                final boolean preview=selected!=null && selected.getBoolean("prerelease");
                final String version=selected==null?installed:selected.getString("tag_name").substring(6);
                final boolean important=selected!=null && supportWarning(selected.optString("body"),installed,version);
                String notes=selected==null?"":selected.optString("body");
                final boolean required=notes.trim().startsWith("<!-- garlic-required-floor:")
                    && supportWarning(notes.replace("garlic-required-floor:", "garlic-support-floor:"),installed,version);
                a.runOnUiThread(() -> {
                    boolean notify=manualRequested;
                    checking=false;
                    manualRequested=false;
                    if (a.isFinishing() || a.isDestroyed()) return;
                    if (version.equals(installed) || (preview && !previews(a))) {
                        if (notify) Toast.makeText(a,"Установлена последняя версия: "+installed,Toast.LENGTH_LONG).show();
                        return;
                    }
                    if(required) {
                        AlertDialog dialog=new AlertDialog.Builder(a).setTitle("Необходимо обновить Garlic")
                            .setIcon(a.getApplicationInfo().icon)
                            .setMessage("Для продолжения работы требуется версия "+version+".")
                            .setCancelable(false).setPositiveButton("Обновить",null).create();
                        dialog.show();
                        dialog.getButton(AlertDialog.BUTTON_POSITIVE).setOnClickListener(v -> {
                            if(!downloading && (!preview || previews(a))) download(a,version,preview);
                        });
                        return;
                    }
                    new AlertDialog.Builder(a).setTitle(important ? "Важное обновление Garlic Alpha" : "Обновление Garlic Alpha")
                      .setMessage((important ? "Поддержка вашей версии скоро закончится. Рекомендуем обновить Garlic.\n\n" : "") + "Скачать и установить " + (preview?"пререлиз ":"релиз ") + version + "?")
                      .setNegativeButton("Позже",null).setPositiveButton("Скачать",(d,w)->{ if(!preview || previews(a)) download(a,version,preview); }).show();
                });
            } catch(Exception e) { android.util.Log.w("Garlic","Release check: "+e.getClass().getSimpleName());
                a.runOnUiThread(() -> {
                    if(manualRequested && !a.isDestroyed()) Toast.makeText(a,"Не удалось проверить GitHub. Попробуйте ещё раз.",Toast.LENGTH_LONG).show();
                    manualRequested=false;
                    checking=false;
                }); }
        },"garlic-release-check").start();
    }
    private static void verify(Activity a, File file, String version) throws Exception {
        PackageManager pm=a.getPackageManager();
        PackageInfo old=pm.getPackageInfo(a.getPackageName(),PackageManager.GET_SIGNING_CERTIFICATES);
        PackageInfo apk=pm.getPackageArchiveInfo(file.getAbsolutePath(),PackageManager.GET_SIGNING_CERTIFICATES);
        if(apk==null) throw new IOException("Android не смог прочитать скачанный APK.");
        if(!old.packageName.equals(apk.packageName)) throw new IOException("APK предназначен для другого приложения.");
        if(apk.getLongVersionCode()<=old.getLongVersionCode()) throw new IOException("Установлена такая же или более новая сборка.");
        if(version!=null && !version.equals(apk.versionName)) throw new IOException("Версия APK отличается от версии релиза.");
        if(old.signingInfo==null || apk.signingInfo==null ||
           !Arrays.equals(old.signingInfo.getApkContentsSigners(),apk.signingInfo.getApkContentsSigners()))
            throw new IOException("Android не подтвердил совпадение подписей приложения.");
    }
    private static void download(Activity a,String version,boolean preview) {
        downloading=true;
        new Thread(()->{
            File part=null;
            String stage="storage";
            try {
                part=File.createTempFile("garlic-update-", ".apk", a.getCacheDir());
                stage="download";
                for(int attempt=0; ; attempt++) {
                    try(OutputStream out=new FileOutputStream(part)) {
                        fetch("https://github.com/ifmain/Garlic/releases/download/alpha-"+version+"/garlic-alpha.apk",out,64*1024*1024);
                        break;
                    } catch(java.net.SocketTimeoutException | EOFException interrupted) {
                        if(attempt!=0) throw interrupted;
                    }
                }
                stage="verify";
                verify(a,part,version);
                if(preview && !previews(a)) return;
                stage="storage";
                java.nio.file.Files.move(part.toPath(),pending(a).toPath(),java.nio.file.StandardCopyOption.REPLACE_EXISTING);
                a.getSharedPreferences("garlic_prefs",0).edit().putBoolean("pending_preview",preview).apply();
                a.runOnUiThread(()->install(a,true));
            } catch(Exception e) {
                String message="download".equals(stage) ? "Не удалось загрузить APK с GitHub." :
                    "storage".equals(stage) ? "Не удалось сохранить APK в памяти телефона." :
                    e instanceof IOException ? e.getMessage() : "Android не смог проверить APK.";
                BackgroundSettings.record(a,"update_error stage="+stage+" version="+version+" error="+e.getClass().getSimpleName()+" detail="+message);
                a.runOnUiThread(()->Toast.makeText(a,message,Toast.LENGTH_LONG).show());
            }
            finally { if(part!=null) part.delete(); a.runOnUiThread(()->downloading=false); }
        },"garlic-release-download").start();
    }
    public static void permissionResult(Activity a,int request) { if(request==PERMISSION) install(a,false); }
    private static void install(Activity a,boolean prompt) {
        try {
            if(a.getSharedPreferences("garlic_prefs",0).getBoolean("pending_preview",false) && !previews(a)) return;
            if(!a.getPackageManager().canRequestPackageInstalls()) {
                if(prompt) a.startActivityForResult(new Intent(Settings.ACTION_MANAGE_UNKNOWN_APP_SOURCES,Uri.parse("package:"+a.getPackageName())),PERMISSION);
                else Toast.makeText(a,"Разрешение на установку не выдано",Toast.LENGTH_LONG).show();
                return;
            }
            verify(a,pending(a),null);
            Uri uri=Uri.parse("content://"+a.getPackageName()+".release/update.apk");
            Intent intent=new Intent(Intent.ACTION_VIEW).setDataAndType(uri,"application/vnd.android.package-archive")
                .addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION);
            a.startActivity(intent);
        } catch(Exception e) { Toast.makeText(a,"Не удалось открыть установщик Android",Toast.LENGTH_LONG).show(); }
    }
}
