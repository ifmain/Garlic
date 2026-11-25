package org.garlic.client;

import android.app.*;
import android.content.*;
import android.widget.Toast;
import android.util.AtomicFile;
import java.io.*;
import java.nio.charset.StandardCharsets;
import org.json.*;

public final class PluginFiles {
    static final int INSTALL = 4851, IMPORT = 4852, EXPORT = 4853;
    static File state(Activity a) { return new File(a.getFilesDir(), "garlic/client.installed-gpf"); }
    static String address(String value) throws JSONException {
        try {
            java.net.URI u=new java.net.URI(value);
            if (!"https".equals(u.getScheme()) || u.getHost()==null || u.getUserInfo()!=null || u.getQuery()!=null ||
                u.getFragment()!=null || !(u.getPath().isEmpty()||u.getPath().equals("/")) || u.getPort()>65535 || u.getPort()==0 || value.length()>240)
                throw new Exception();
            return value.endsWith("/")?value.substring(0,value.length()-1):value;
        } catch(Exception e){throw new JSONException("address");}
    }
    static JSONObject parse(String text, boolean manifest) throws JSONException {
        JSONTokener input = new JSONTokener(text);
        Object item = input.nextValue();
        if (!(item instanceof JSONObject) || input.nextClean() != 0) throw new JSONException("JSON");
        JSONObject v = (JSONObject)item;
        if (v.length() != (manifest ? 4 : (v.has("connections") ? 6 : 5)) || !"dev-connection".equals(v.get("id")) ||
            !(manifest ? "gpf/1" : "garlic-plugin-data/1").equals(v.get("format")) ||
            (!manifest && !(v.get("enabled") instanceof Boolean))) throw new JSONException("Format");
        for (String key : new String[]{"interval_seconds","ingress_penalty_ms"}) {
            Object n = v.get(key);
            boolean interval = key.equals("interval_seconds");
            if (!(n instanceof Integer) || (Integer)n < (interval ? 3 : 0) ||
                (Integer)n > (interval ? 60 : 250)) throw new JSONException(key);
        }
        JSONArray rows=v.has("connections")?v.getJSONArray("connections"):new JSONArray();
        if(rows.length()>24)throw new JSONException("connections");
        for(int i=0;i<rows.length();i++){
            JSONObject row=rows.getJSONObject(i);
            if(row.length()!=2 || !("direct".equals(row.get("kind")) || "ingress".equals(row.get("kind")))) throw new JSONException("kind");
            row.put("address",address(row.getString("address")));
        }
        return v.put("connections",rows).put("format","garlic-plugin-data/1").put("enabled",manifest || v.getBoolean("enabled"));
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
        return parse(read(file.openRead()),false);
    }
    static void save(Activity a, JSONObject data) throws Exception {
        AtomicFile file = new AtomicFile(state(a));
        FileOutputStream out = file.startWrite();
        try { out.write(data.toString().getBytes(StandardCharsets.UTF_8)); file.finishWrite(out); }
        catch (Exception e) { file.failWrite(out); throw e; }
    }
    static void show(Activity a) {
        a.startActivity(new Intent(a,PluginPage.class));
    }
    static void result(Activity a,int request,int result,Intent data) {
        if(request<INSTALL || request>EXPORT || result!=Activity.RESULT_OK || data==null || data.getData()==null) return;
        new Thread(() -> {
            try {
                if(request!=INSTALL && !state(a).isFile()) throw new IOException();
                if(request==INSTALL && state(a).isFile()) throw new IOException();
                if(request==EXPORT) {
                    byte[] bytes=load(a).toString(2).getBytes(StandardCharsets.UTF_8);
                    try(OutputStream out=a.getContentResolver().openOutputStream(data.getData(),"wt")) {
                        if(out==null) throw new IOException();
                        out.write(bytes);
                    }
                } else {
                    save(a,parse(read(a.getContentResolver().openInputStream(data.getData())),request==INSTALL));
                }
                a.runOnUiThread(() -> { if(a instanceof PluginPage)((PluginPage)a).render(); toast(a,request==EXPORT?"Данные сохранены":"Готово"); });
            } catch(Exception e) { a.runOnUiThread(() -> toast(a,"Ошибка файла плагина")); }
        },"garlic-plugin-file").start();
    }
    static void toast(Activity a,String text) { Toast.makeText(a,text,Toast.LENGTH_LONG).show(); }
}
