package org.garlic.client;

import android.app.*;
import android.content.*;
import android.widget.Toast;
import android.util.AtomicFile;
import java.io.*;
import java.nio.charset.StandardCharsets;
import org.json.*;

public final class PluginFiles {
    private static final int INSTALL = 4851, IMPORT = 4852, EXPORT = 4853;
    static File state(Activity a) { return new File(a.getFilesDir(), "garlic/client.plugin-data"); }
    static JSONObject defaults() throws JSONException {
        return new JSONObject().put("id","dev-connection").put("format","garlic-plugin-data/1")
            .put("enabled",false).put("interval_seconds",5).put("ingress_penalty_ms",0);
    }
    static JSONObject parse(String text, boolean manifest) throws JSONException {
        JSONTokener input = new JSONTokener(text);
        Object item = input.nextValue();
        if (!(item instanceof JSONObject) || input.nextClean() != 0) throw new JSONException("JSON");
        JSONObject v = (JSONObject)item;
        if (v.length() != (manifest ? 4 : 5) || !"dev-connection".equals(v.get("id")) ||
            !(manifest ? "gpf/1" : "garlic-plugin-data/1").equals(v.get("format")) ||
            (!manifest && !(v.get("enabled") instanceof Boolean))) throw new JSONException("Format");
        for (String key : new String[]{"interval_seconds","ingress_penalty_ms"}) {
            Object n = v.get(key);
            boolean interval = key.equals("interval_seconds");
            if (!(n instanceof Integer) || (Integer)n < (interval ? 3 : 0) ||
                (Integer)n > (interval ? 60 : 250)) throw new JSONException(key);
        }
        return v.put("format","garlic-plugin-data/1").put("enabled",manifest || v.getBoolean("enabled"));
    }
    static String read(InputStream in) throws IOException {
        try (InputStream stream = in; ByteArrayOutputStream bytes = new ByteArrayOutputStream()) {
            if (stream == null) throw new IOException();
            byte[] buf = new byte[2048]; int n;
            while ((n = stream.read(buf)) != -1) {
                if (bytes.size()+n > 8192) throw new IOException();
                bytes.write(buf,0,n);
            }
            return bytes.toString("UTF-8");
        }
    }
    static JSONObject load(Activity a) throws Exception {
        AtomicFile file = new AtomicFile(state(a));
        try { return parse(read(file.openRead()),false); }
        catch (FileNotFoundException e) { return defaults(); }
    }
    static void save(Activity a, JSONObject data) throws Exception {
        AtomicFile file = new AtomicFile(state(a));
        FileOutputStream out = file.startWrite();
        try { out.write(data.toString().getBytes(StandardCharsets.UTF_8)); file.finishWrite(out); }
        catch (Exception e) { file.failWrite(out); throw e; }
    }
    static void show(Activity a) {
        try {
            JSONObject current = load(a);
            boolean enabled = current.getBoolean("enabled");
            new AlertDialog.Builder(a).setTitle("Плагины").setItems(new String[]{
                "Dev Connection · " + (enabled ? "выключить" : "включить"), "Установить GPF",
                "Импорт данных (JSON)", "Экспорт данных (JSON)"}, (d,i) -> {
                try {
                    if (i==0) { save(a,current.put("enabled",!enabled)); toast(a,"Применится при следующем подключении"); return; }
                    boolean export = i==3;
                    Intent intent = new Intent(export ? Intent.ACTION_CREATE_DOCUMENT : Intent.ACTION_OPEN_DOCUMENT);
                    intent.addCategory(Intent.CATEGORY_OPENABLE).setType(export ? "application/json" : "*/*");
                    if (export) intent.putExtra(Intent.EXTRA_TITLE,"dev-connection.json");
                    a.startActivityForResult(intent,i==1 ? INSTALL : export ? EXPORT : IMPORT);
                } catch(Exception e) { toast(a,"Не удалось выполнить действие"); }
            }).show();
        } catch(Exception e) { toast(a,"Не удалось прочитать настройки плагина"); }
    }
    static void result(Activity a,int request,int result,Intent data) {
        if(request<INSTALL || request>EXPORT || result!=Activity.RESULT_OK || data==null || data.getData()==null) return;
        new Thread(() -> {
            try {
                if(request==EXPORT) {
                    byte[] bytes=load(a).toString(2).getBytes(StandardCharsets.UTF_8);
                    try(OutputStream out=a.getContentResolver().openOutputStream(data.getData(),"wt")) {
                        if(out==null) throw new IOException();
                        out.write(bytes);
                    }
                } else {
                    save(a,parse(read(a.getContentResolver().openInputStream(data.getData())),request==INSTALL));
                }
                a.runOnUiThread(() -> toast(a,request==EXPORT ? "Данные сохранены" : "Применится при следующем подключении"));
            } catch(Exception e) { a.runOnUiThread(() -> toast(a,"Ошибка файла плагина")); }
        },"garlic-plugin-file").start();
    }
    static void toast(Activity a,String text) { Toast.makeText(a,text,Toast.LENGTH_LONG).show(); }
}
