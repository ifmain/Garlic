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
    private static void fetch(String address, OutputStream out, int limit) throws IOException {
        HttpsURLConnection c = (HttpsURLConnection)new URL(address).openConnection();
        c.setRequestProperty("User-Agent", "Garlic-Alpha");
        c.setConnectTimeout(10000); c.setReadTimeout(20000);
        try {
            if(c.getResponseCode()!=200 || c.getContentLengthLong()>limit) throw new IOException("Download failed");
            try(InputStream in=c.getInputStream()) {
                byte[] buffer=new byte[32768]; int size=0,n;
                while((n=in.read(buffer))!=-1) {
                    size+=n; if(size>limit) throw new IOException("Download too large");
                    out.write(buffer,0,n);
                }
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
    public static void addButton(Activity a, android.widget.LinearLayout parent) {
        android.widget.Button button = new android.widget.Button(a);
        button.setText("Обновить с GitHub");
        button.setOnClickListener(v -> check(a, true));
        parent.addView(button);
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
                String best=installed;
                ByteArrayOutputStream out=new ByteArrayOutputStream();
                fetch("https://api.github.com/repos/ifmain/Garlic/releases?per_page=30",out,1024*1024);
                JSONArray items=new JSONArray(out.toString("UTF-8"));
                for(int i=0;i<items.length();i++) {
                    JSONObject item=items.getJSONObject(i);String tag=item.getString("tag_name");
                    if(!item.getBoolean("draft") && !item.getBoolean("prerelease") && tag.startsWith("alpha-") && newer(tag.substring(6),best)) best=tag.substring(6);
                }
                final String version=best;
                a.runOnUiThread(() -> {
                    boolean notify=manualRequested;
                    checking=false;
                    manualRequested=false;
                    if (a.isFinishing() || a.isDestroyed()) return;
                    if (version.equals(installed)) {
                        if (notify) Toast.makeText(a,"Установлена последняя версия: "+installed,Toast.LENGTH_LONG).show();
                        return;
                    }
                    new AlertDialog.Builder(a).setTitle("Обновление Garlic Alpha")
                      .setMessage("Скачать и установить версию " + version + "?")
                      .setNegativeButton("Позже",null).setPositiveButton("Скачать",(d,w)->download(a,version)).show();
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
        if(apk==null || !old.packageName.equals(apk.packageName) || apk.getLongVersionCode()<=old.getLongVersionCode() ||
           (version!=null && !version.equals(apk.versionName)) || old.signingInfo==null || apk.signingInfo==null ||
           !Arrays.equals(old.signingInfo.getApkContentsSigners(),apk.signingInfo.getApkContentsSigners())) throw new IOException("Package verification failed");
    }
    private static void download(Activity a,String version) {
        downloading=true;
        new Thread(()->{
            File part=null;
            try {
                part=File.createTempFile("garlic-update-", ".part", a.getCacheDir());
                try(OutputStream out=new FileOutputStream(part)) {
                    fetch("https://github.com/ifmain/Garlic/releases/download/alpha-"+version+"/garlic-alpha.apk",out,64*1024*1024);
                }
                verify(a,part,version);
                java.nio.file.Files.move(part.toPath(),pending(a).toPath(),java.nio.file.StandardCopyOption.REPLACE_EXISTING);
                a.runOnUiThread(()->install(a,true));
            } catch(Exception e) { a.runOnUiThread(()->Toast.makeText(a,"Не удалось скачать или проверить обновление",Toast.LENGTH_LONG).show()); }
            finally { if(part!=null) part.delete(); a.runOnUiThread(()->downloading=false); }
        },"garlic-release-download").start();
    }
    public static void permissionResult(Activity a,int request) { if(request==PERMISSION) install(a,false); }
    private static void install(Activity a,boolean prompt) {
        try {
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
