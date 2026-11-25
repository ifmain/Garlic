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
                button(page,(row.getString("kind").equals("direct")?"Напрямую":"Через входную ноду")+"   ·   Изменить",()->editor(index));}
            button(page,"＋  Добавить подключение",()->editor(-1));
            }
            label("Данные плагина",20);button(page,"Импорт JSON",()->pick(PluginFiles.IMPORT));button(page,"Экспорт JSON",()->pick(PluginFiles.EXPORT));
            button(page,"Удалить плагин",()->new AlertDialog.Builder(this).setMessage("Удалить плагин и его настройки?")
                .setNegativeButton("Отмена",null).setPositiveButton("Удалить",(d,w)->{new java.io.File(PluginFiles.folder(this),"plugin/client.sessions.json").delete();new android.util.AtomicFile(PluginFiles.module(this)).delete();new android.util.AtomicFile(PluginFiles.state(this)).delete();details=false;selected=null;render();}).show());
        }catch(Exception e){label("Не удалось прочитать настройки",16);button(page,"Восстановить из JSON",()->pick(PluginFiles.IMPORT));}
    }
    void editor(int index) throws Exception {
        JSONArray rows=config.getJSONArray("connections"); JSONObject old=index<0?null:rows.getJSONObject(index);
        LinearLayout box=new LinearLayout(this);box.setOrientation(1);box.setPadding(dp(20),dp(8),dp(20),0);
        Spinner kind=new Spinner(this);ArrayAdapter<String> types=new ArrayAdapter<>(this,android.R.layout.simple_spinner_dropdown_item,new String[]{"Garlic · напрямую","Garlic · входная нода"});kind.setAdapter(types);box.addView(kind);
        EditText addr=new EditText(this);addr.setSingleLine();addr.setInputType(17);addr.setHint("https://server.example:8443");box.addView(addr);
        EditText key=new EditText(this);key.setSingleLine();key.setInputType(129);key.setSaveEnabled(false);key.setHint("Ключ Garlic · пусто для сохранённого");box.addView(key);
        if(old!=null){addr.setText(old.getString("address"));kind.setSelection(old.getString("kind").equals("direct")?0:1);}
        AlertDialog.Builder dlg=new AlertDialog.Builder(this).setTitle(index<0?"Добавить подключение":"Подключение").setView(box).setNegativeButton("Отмена",null).setPositiveButton("Сохранить",null);
        if(index>=0)dlg.setNeutralButton("Удалить",(d,w)->{rows.remove(index);try{write();}catch(Exception e){render();}});
        AlertDialog d=dlg.show();d.getButton(-1).setOnClickListener(v->{try{
            String address=PluginFiles.address(addr.getText().toString().trim());
            if(index<0 && rows.length()>=24)throw new Exception();
            if(key.length()>0) {
                java.io.File dir=new java.io.File(PluginFiles.folder(this),"plugin");
                Object reply=Class.forName("org.garlic.client.GarlicBridge").getMethod("saveKey",String.class,String.class,String.class)
                    .invoke(null,dir.getAbsolutePath(),address,key.getText().toString());
                if(!new JSONObject((String)reply).optBoolean("ok"))throw new Exception();
                key.setText("");
            }
            rows.put(index<0?rows.length():index,new JSONObject().put("kind",kind.getSelectedItemPosition()==0?"direct":"ingress").put("address",address));
            write();d.dismiss();
        }catch(Exception e){addr.setError("Введите HTTPS-адрес сервера без ключа, пути и параметров");}});
    }
}
