package garlic.lab;

import android.app.*;
import android.content.*;
import android.os.*;
import android.view.*;
import android.widget.*;
import java.io.*;
import java.nio.file.*;
import java.util.*;
import org.json.*;

/** Instrumentation for a fresh emulator. The signed test APK supplies disposable slot data. */
public final class PluginSettingsProbe extends Instrumentation {
    private Activity page;
    private final java.util.concurrent.CountDownLatch initialized=new java.util.concurrent.CountDownLatch(1);
    public void onCreate(Bundle extras){super.onCreate(extras);start();}
    public void callApplicationOnCreate(Application app){super.callApplicationOnCreate(app);initialized.countDown();}
    private void buttons(View view,List<Button> out){
        if(view instanceof Button)out.add((Button)view);
        if(view instanceof ViewGroup){ViewGroup g=(ViewGroup)view;for(int i=0;i<g.getChildCount();i++)buttons(g.getChildAt(i),out);}
    }
    private List<Button> matching(String text){
        List<Button> all=new ArrayList<>();buttons(page.getWindow().getDecorView(),all);
        all.removeIf(b->!text.contentEquals(b.getText()));return all;
    }
    public void onStart(){
        Bundle result=new Bundle();Path slot=null;
        try {
            if(!initialized.await(90,java.util.concurrent.TimeUnit.SECONDS))throw new IOException("Application startup timed out");
            File base=new File(getTargetContext().getFilesDir(),"garlic");
            slot=base.toPath().resolve("plugins-v3/s-dev-connection");
            Files.createDirectories(slot.getParent());Files.createDirectory(slot);
            for(String name:getContext().getAssets().list("fixture")){
                try(InputStream in=getContext().getAssets().open("fixture/"+name)){Files.copy(in,slot.resolve(name));}
            }
            Intent intent=new Intent().setClassName(getTargetContext(),"org.garlic.client.PluginPage").addFlags(Intent.FLAG_ACTIVITY_NEW_TASK);
            page=startActivitySync(intent);
            final List<Button> probes=new ArrayList<>();
            runOnMainSync(()->{matching("⚙  Настройки").get(0).performClick();probes.addAll(matching("Протестировать"));});
            if(probes.size()!=6)throw new IOException("expected six fixture rows, got "+probes.size());
            for(int i=0;i<probes.size();i++){
                Button b=probes.get(i);runOnMainSync(()->b.performClick());String[] text={""};
                long until=SystemClock.elapsedRealtime()+30000;
                do{
                    Thread.sleep(150);runOnMainSync(()->text[0]=b.isEnabled()?b.getText().toString():"");
                }while(text[0].isEmpty() && SystemClock.elapsedRealtime()<until);
                if(i<5 ? !text[0].equals("● Работает") : !text[0].startsWith("● Ошибка"))throw new IOException("row "+i+": "+text[0]);
                result.putString("row"+i,text[0]);
            }
            finish(-1,result);
        }catch(Throwable e){result.putString("error",android.util.Log.getStackTraceString(e));finish(0,result);}
        finally {
            if(page!=null)runOnMainSync(()->page.finish());
            if(slot!=null && result.containsKey("row0"))try{
                for(String name:getContext().getAssets().list("fixture"))Files.deleteIfExists(slot.resolve(name));
                Files.deleteIfExists(slot);
            }catch(IOException ignored){}
        }
    }
}
