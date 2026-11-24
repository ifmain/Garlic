.class Lorg/garlic/client/InterfaceFactory$1;
.super Ljava/lang/Object;
.source "InterfaceFactory.java"

# interfaces
.implements Landroid/app/Application$ActivityLifecycleCallbacks;


# annotations
.annotation system Ldalvik/annotation/EnclosingMethod;
    value = Lorg/garlic/client/InterfaceFactory;->confirmOnResume(Landroid/app/Application;J)V
.end annotation

.annotation system Ldalvik/annotation/InnerClass;
    accessFlags = 0x0
    name = null
.end annotation


# instance fields
.field final synthetic val$app:Landroid/app/Application;

.field final synthetic val$sequence:J


# direct methods
.method constructor <init>(Landroid/app/Application;J)V
    .locals 0
    .annotation system Ldalvik/annotation/Signature;
        value = {
            "()V"
        }
    .end annotation

    .line 55
    iput-object p1, p0, Lorg/garlic/client/InterfaceFactory$1;->val$app:Landroid/app/Application;

    iput-wide p2, p0, Lorg/garlic/client/InterfaceFactory$1;->val$sequence:J

    invoke-direct {p0}, Ljava/lang/Object;-><init>()V

    return-void
.end method


# virtual methods
.method public onActivityCreated(Landroid/app/Activity;Landroid/os/Bundle;)V
    .locals 0

    .line 63
    return-void
.end method

.method public onActivityDestroyed(Landroid/app/Activity;)V
    .locals 0

    .line 68
    return-void
.end method

.method public onActivityPaused(Landroid/app/Activity;)V
    .locals 0

    .line 65
    return-void
.end method

.method public onActivityResumed(Landroid/app/Activity;)V
    .locals 2

    .line 57
    invoke-virtual {p1}, Ljava/lang/Object;->getClass()Ljava/lang/Class;

    move-result-object p1

    invoke-virtual {p1}, Ljava/lang/Class;->getClassLoader()Ljava/lang/ClassLoader;

    move-result-object p1

    invoke-static {}, Lorg/garlic/client/InterfaceFactory;->-$$Nest$sfgetloader()Ljava/lang/ClassLoader;

    move-result-object v0

    if-eq p1, v0, :cond_0

    return-void

    .line 59
    :cond_0
    :try_start_0
    iget-object p1, p0, Lorg/garlic/client/InterfaceFactory$1;->val$app:Landroid/app/Application;

    iget-wide v0, p0, Lorg/garlic/client/InterfaceFactory$1;->val$sequence:J

    invoke-static {v0, v1}, Ljava/lang/Long;->toString(J)Ljava/lang/String;

    move-result-object v0

    invoke-static {p1, v0}, Lorg/garlic/client/CoreLoader;->interfaceUpdate(Landroid/content/Context;Ljava/lang/String;)Lorg/json/JSONObject;

    move-result-object p1

    const-string v0, "ok"

    invoke-virtual {p1, v0}, Lorg/json/JSONObject;->optBoolean(Ljava/lang/String;)Z

    move-result p1

    if-eqz p1, :cond_1

    .line 60
    iget-object p1, p0, Lorg/garlic/client/InterfaceFactory$1;->val$app:Landroid/app/Application;

    invoke-virtual {p1, p0}, Landroid/app/Application;->unregisterActivityLifecycleCallbacks(Landroid/app/Application$ActivityLifecycleCallbacks;)V
    :try_end_0
    .catch Ljava/lang/Exception; {:try_start_0 .. :try_end_0} :catch_0

    .line 61
    :cond_1
    goto :goto_0

    :catch_0
    move-exception p1

    const-string v0, "Garlic"

    const-string v1, "Interface confirmation failed"

    invoke-static {v0, v1, p1}, Landroid/util/Log;->w(Ljava/lang/String;Ljava/lang/String;Ljava/lang/Throwable;)I

    .line 62
    :goto_0
    return-void
.end method

.method public onActivitySaveInstanceState(Landroid/app/Activity;Landroid/os/Bundle;)V
    .locals 0

    .line 67
    return-void
.end method

.method public onActivityStarted(Landroid/app/Activity;)V
    .locals 0

    .line 64
    return-void
.end method

.method public onActivityStopped(Landroid/app/Activity;)V
    .locals 0

    .line 66
    return-void
.end method
