package org.garlic.client;

import android.app.*;
import android.content.*;
import android.graphics.Color;
import android.graphics.drawable.GradientDrawable;
import android.os.Bundle;
import android.view.*;
import android.widget.*;
import org.json.*;

public final class PluginPage extends Activity {
    static boolean builtin(String k){return k.equals("direct")||k.equals("ingress");}
    static String display(String k){return k.equals("direct")?"Garlic · напрямую":k.equals("ingress")?"Garlic · входная нода":k.toUpperCase(java.util.Locale.ROOT);}
    private static native String testPeer(String dir,String slot,String kind,String address);
    private LinearLayout page;
    String selected;
    private boolean details;
    private JSONObject config;
    int dp(int n) { return Math.round(getResources().getDisplayMetrics().density*n); }
    @Override public void onCreate(Bundle state) { super.onCreate(state); selected=state==null?null:state.getString("slot"); details=state!=null && state.getBoolean("details"); render(); }
    @Override public void onSaveInstanceState(Bundle b) { super.onSaveInstanceState(b); b.putBoolean("details",details); b.putString("slot",selected); }
    @Override public void onBackPressed() { if(details){details=false; render();} else super.onBackPressed(); }
    @Override protected void onActivityResult(int request,int result,Intent data) {
        super.onActivityResult(request,result,data); PluginFiles.result(this,request,result,data);
    }
    void label(String s,int size) { TextView t=new TextView(this); t.setText(s); t.setTextSize(size); t.setTextColor(0xffeeeeee); t.setPadding(0,dp(12),0,dp(8)); page.addView(t); }
    interface Action { void run() throws Exception; }
    Button button(LinearLayout parent,String title,Action action) {
        Button b=new Button(this); b.setText(title); b.setAllCaps(false); b.setTextColor(Color.WHITE);
        GradientDrawable bg=new GradientDrawable(); bg.setColor(0xff292929); bg.setCornerRadius(dp(12)); b.setBackground(bg);
        LinearLayout.LayoutParams lp=new LinearLayout.LayoutParams(parent==page?-1:-2,dp(48)); lp.setMargins(0,dp(4),dp(6),dp(4));parent.addView(b,lp);
        b.setOnClickListener(v->{try{action.run();}catch(Exception e){PluginFiles.toast(this,"Проверьте данные подключения");}});return b;
    }
    void write() throws Exception { PluginFiles.save(this,config); render(); }
    void pick(int code) {
        boolean export=code==PluginFiles.EXPORT;
        Intent i=new Intent(export?Intent.ACTION_CREATE_DOCUMENT:Intent.ACTION_OPEN_DOCUMENT);
        i.addCategory(Intent.CATEGORY_OPENABLE).setType(export?"application/json":"*/*");
        if(export)i.putExtra(Intent.EXTRA_TITLE,"plugin-data.json"); startActivityForResult(i,code);
    }
    void render() {
        ScrollView scroll=new ScrollView(this);scroll.setBackgroundColor(0xff181818);page=new LinearLayout(this);page.setOrientation(1);
        page.setPadding(dp(20),dp(24),dp(20),dp(24));scroll.addView(page);setContentView(scroll);
        button(page,details?"‹  Плагины":"‹  Garlic",()->onBackPressed());
        if(!details){
            selected=null;label("Плагины",28);int shown=0;
            java.util.List<String> ids=new java.util.ArrayList<>();ids.add(null);
            java.io.File[] dirs=new java.io.File(PluginFiles.folder(this,null),"plugins-v3").listFiles();
            if(dirs!=null){java.util.Arrays.sort(dirs);for(java.io.File d:dirs)if(d.getName().matches("s-[a-z0-9-]{1,48}"))ids.add(d.getName().substring(2));}
            for(String id:ids){try{
                selected=id;if(!PluginFiles.module(this).isFile())continue;
                JSONObject settings=PluginFiles.load(this);label(PluginFiles.title(this),21);shown++;
                LinearLayout row=new LinearLayout(this);page.addView(row);Switch toggle=new Switch(this);toggle.setText("Включён   ");
                toggle.setChecked(settings.getBoolean("enabled"));row.addView(toggle);
                toggle.setOnCheckedChangeListener((v,on)->{try{selected=id;PluginFiles.save(this,settings.put("enabled",on));}catch(Exception e){PluginFiles.toast(this,"Ошибка сохранения");}finally{selected=null;render();}});
                button(row,"⚙  Настройки",()->{selected=id;details=true;render();});
            }catch(Exception e){PluginFiles.toast(this,"Ошибка чтения слота");}}
            selected=null;if(shown==0)label("Нет установленных плагинов",16);
            button(page,"Установить GPF",()->pick(PluginFiles.INSTALL));return;
        }
        try{
            config=PluginFiles.load(this);JSONObject m=PluginFiles.manifest(this);label(m.getString("name"),28);
            if("canvas".equals(m.optString("capability"))){
                if(config.getBoolean("enabled"))page.addView(new ModuleCanvas(this,m.getString("wasm")),new LinearLayout.LayoutParams(-1,dp(280)));
                else label("Плагин выключен",16);
            }else{
            label("Подключения",20);label("Используются при следующем подключении.",14);
            JSONArray rows=config.getJSONArray("connections");
            for(int i=0;i<rows.length();i++){final int index=i;JSONObject row=rows.getJSONObject(i);label(row.getString("address"),16);
                String type=row.getString("kind");button(page,display(type)+"   ·   Изменить",()->editor(index));
                if(!builtin(type)){
                    final String slot=selected,dir=PluginFiles.folder(this,null).getAbsolutePath(),endpoint=row.getString("address");
                    Button probe=button(page,"Протестировать",()->{});probe.setOnClickListener(v->{probe.setEnabled(false);probe.setText("Проверяем…");
                        new Thread(()->{String error=null;try{System.loadLibrary("garlic_internal_network");JSONObject result=new JSONObject(testPeer(dir,slot,type,endpoint));
                            if(!result.optBoolean("ok"))error=result.optString("error","Ошибка подключения");
                        }catch(Exception|LinkageError e){error="Сетевой модуль недоступен";}
                        final String why=error;runOnUiThread(()->{probe.setEnabled(true);probe.setText(why==null?"● Работает":"● Ошибка: "+why);probe.setTextColor(why==null?0xff86db99:0xffff9085);});
                        },"plugin-test").start();});
                }
            }
            button(page,"＋  Добавить подключение",()->editor(-1));
            }
            label("Данные плагина",20);button(page,"Импорт JSON",()->pick(PluginFiles.IMPORT));button(page,"Экспорт JSON",()->pick(PluginFiles.EXPORT));
            button(page,"Удалить плагин",()->new AlertDialog.Builder(this).setMessage("Удалить плагин и его настройки?")
                .setNegativeButton("Отмена",null).setPositiveButton("Удалить",(d,w)->{new java.io.File(PluginFiles.folder(this),"plugin/client.sessions.json").delete();new android.util.AtomicFile(new java.io.File(PluginFiles.folder(this),"client.connector-login")).delete();new android.util.AtomicFile(PluginFiles.module(this)).delete();new android.util.AtomicFile(PluginFiles.state(this)).delete();details=false;selected=null;render();}).show());
        }catch(Exception e){label("Не удалось прочитать настройки",16);button(page,"Восстановить из JSON",()->pick(PluginFiles.IMPORT));}
    }
    void editor(int index) throws Exception {
        JSONArray rows=config.getJSONArray("connections"); JSONObject old=index<0?null:rows.getJSONObject(index);
        LinearLayout box=new LinearLayout(this);box.setOrientation(1);box.setPadding(dp(20),dp(8),dp(20),0);
        JSONArray allowed=PluginFiles.manifest(this).optJSONArray("transports");
        java.util.ArrayList<String> kinds=new java.util.ArrayList<>(),names=new java.util.ArrayList<>();
        kinds.add("direct");kinds.add("ingress");
        for(int i=0;allowed!=null && i<allowed.length();i++){String k=allowed.getString(i);if(!kinds.contains(k))kinds.add(k);}
        for(String k:kinds)names.add(display(k));
        android.util.AtomicFile secretFile=new android.util.AtomicFile(new java.io.File(PluginFiles.folder(this),"client.connector-login"));
        JSONObject secrets=secretFile.getBaseFile().exists()?new JSONObject(PluginFiles.read(secretFile.openRead())):new JSONObject();
        Spinner kind=new Spinner(this);ArrayAdapter<String> types=new ArrayAdapter<>(this,android.R.layout.simple_spinner_dropdown_item,names);kind.setAdapter(types);box.addView(kind);
        EditText addr=new EditText(this);addr.setSingleLine();addr.setInputType(17);addr.setHint("https://server.example:8443");box.addView(addr);
        EditText user=new EditText(this);user.setSingleLine();user.setHint("Пользователь прокси / SSH");box.addView(user);
        EditText pin=new EditText(this);pin.setSingleLine();pin.setHint("SSH: SHA256:… · HTTPS: свой CA (DER в Base64)");box.addView(pin);
        EditText key=new EditText(this);key.setSingleLine();key.setInputType(129);key.setSaveEnabled(false);key.setHint("Ключ Garlic или пароль · пусто для сохранённого");box.addView(key);
        if(old!=null){addr.setText(old.getString("address"));kind.setSelection(kinds.indexOf(old.getString("kind")));JSONObject login=secrets.optJSONObject(old.getString("address"));if(login!=null){user.setText(login.optString("user"));pin.setText(login.optString("pin"));}}
        AlertDialog.Builder dlg=new AlertDialog.Builder(this).setTitle(index<0?"Добавить подключение":"Подключение").setView(box).setNegativeButton("Отмена",null).setPositiveButton("Сохранить",null);
        if(index>=0)dlg.setNeutralButton("Удалить",(d,w)->{rows.remove(index);try{write();}catch(Exception e){render();}});
        AlertDialog d=dlg.show();d.getButton(-1).setOnClickListener(v->{try{
            String type=kinds.get(kind.getSelectedItemPosition());String address=PluginFiles.address(addr.getText().toString().trim(),type);
            if(index<0 && rows.length()>=24)throw new Exception();
            if(kind.getSelectedItemPosition()>=2){
                JSONObject prior=secrets.optJSONObject(address);String name=user.getText().toString();
                String password=key.length()>0?key.getText().toString():prior!=null&&name.equals(prior.optString("user"))?prior.optString("password"):"";
                secrets.put(address,new JSONObject().put("user",name).put("pin",pin.getText().toString().trim()).put("password",password));
                java.io.FileOutputStream out=secretFile.startWrite();try{out.write(secrets.toString().getBytes(java.nio.charset.StandardCharsets.UTF_8));secretFile.finishWrite(out);}catch(Exception e){secretFile.failWrite(out);throw e;}
            }else if(key.length()>0) {
                java.io.File dir=new java.io.File(PluginFiles.folder(this),"plugin");
                Object reply=Class.forName("org.garlic.client.GarlicBridge").getMethod("saveKey",String.class,String.class,String.class)
                    .invoke(null,dir.getAbsolutePath(),address,key.getText().toString());
                if(!new JSONObject((String)reply).optBoolean("ok"))throw new Exception();
                key.setText("");
            }
            rows.put(index<0?rows.length():index,new JSONObject().put("kind",type).put("address",address));
            write();d.dismiss();
        }catch(Exception e){addr.setError("Укажите адрес со схемой выбранного протокола и портом");}});
    }
}
