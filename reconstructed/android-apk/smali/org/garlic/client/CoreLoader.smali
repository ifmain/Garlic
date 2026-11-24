.class public final Lorg/garlic/client/CoreLoader;
.super Ljava/lang/Object;
.source "CoreLoader.java"


# static fields
.field private static final checking:Ljava/util/concurrent/atomic/AtomicBoolean;

.field public static volatile version:Ljava/lang/String;


# direct methods
.method static constructor <clinit>()V
    .locals 1

    .line 10
    new-instance v0, Ljava/util/concurrent/atomic/AtomicBoolean;

    invoke-direct {v0}, Ljava/util/concurrent/atomic/AtomicBoolean;-><init>()V

    sput-object v0, Lorg/garlic/client/CoreLoader;->checking:Ljava/util/concurrent/atomic/AtomicBoolean;

    return-void
.end method

.method public constructor <init>()V
    .locals 0

    .line 8
    invoke-direct {p0}, Ljava/lang/Object;-><init>()V

    return-void
.end method

.method private static native bind(Ljava/lang/String;Ljava/lang/String;)Z
.end method

.method public static check(Landroid/content/Context;)V
    .locals 3

    .line 39
    sget-object v0, Lorg/garlic/client/GarlicBridge;->loadError:Ljava/lang/String;

    if-nez v0, :cond_1

    sget-object v0, Lorg/garlic/client/CoreLoader;->checking:Ljava/util/concurrent/atomic/AtomicBoolean;

    const/4 v1, 0x0

    const/4 v2, 0x1

    invoke-virtual {v0, v1, v2}, Ljava/util/concurrent/atomic/AtomicBoolean;->compareAndSet(ZZ)Z

    move-result v0

    if-nez v0, :cond_0

    goto :goto_0

    .line 40
    :cond_0
    invoke-static {p0}, Lorg/garlic/client/CoreLoader;->directory(Landroid/content/Context;)Ljava/lang/String;

    move-result-object p0

    .line 41
    new-instance v0, Ljava/lang/Thread;

    new-instance v1, Lorg/garlic/client/CoreLoader$0;

    invoke-direct {v1, p0}, Lorg/garlic/client/CoreLoader$0;-><init>(Ljava/lang/String;)V

    const-string p0, "Garlic update"

    invoke-direct {v0, v1, p0}, Ljava/lang/Thread;-><init>(Ljava/lang/Runnable;Ljava/lang/String;)V

    .line 44
    invoke-virtual {v0}, Ljava/lang/Thread;->start()V

    .line 45
    return-void

    .line 39
    :cond_1
    :goto_0
    return-void
.end method

.method private static directory(Landroid/content/Context;)Ljava/lang/String;
    .locals 2

    .line 14
    new-instance v0, Ljava/io/File;

    invoke-virtual {p0}, Landroid/content/Context;->getFilesDir()Ljava/io/File;

    move-result-object p0

    const-string v1, "garlic"

    invoke-direct {v0, p0, v1}, Ljava/io/File;-><init>(Ljava/io/File;Ljava/lang/String;)V

    invoke-virtual {v0}, Ljava/io/File;->getAbsolutePath()Ljava/lang/String;

    move-result-object p0

    return-object p0
.end method

.method static initialize(Landroid/content/Context;)V
    .locals 5

    .line 18
    const-string v0, "version"

    const-string v1, "path"

    :try_start_0
    const-string v2, "garlic_bind"

    invoke-static {v2}, Ljava/lang/System;->loadLibrary(Ljava/lang/String;)V

    .line 19
    const-string v2, "garlic_updates"

    invoke-static {v2}, Ljava/lang/System;->loadLibrary(Ljava/lang/String;)V

    .line 20
    invoke-static {p0}, Lorg/garlic/client/CoreLoader;->directory(Landroid/content/Context;)Ljava/lang/String;

    move-result-object v2

    .line 21
    new-instance v3, Lorg/json/JSONObject;

    const-string v4, "select"

    invoke-static {v2, v4}, Lorg/garlic/client/CoreLoader;->update(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;

    move-result-object v4

    invoke-direct {v3, v4}, Lorg/json/JSONObject;-><init>(Ljava/lang/String;)V

    .line 22
    invoke-virtual {v3, v1}, Lorg/json/JSONObject;->has(Ljava/lang/String;)Z

    move-result v4

    if-eqz v4, :cond_1

    invoke-virtual {v3, v1}, Lorg/json/JSONObject;->getString(Ljava/lang/String;)Ljava/lang/String;

    move-result-object v1

    invoke-virtual {v3, v0}, Lorg/json/JSONObject;->getString(Ljava/lang/String;)Ljava/lang/String;

    move-result-object v4

    invoke-static {v1, v4}, Lorg/garlic/client/CoreLoader;->bind(Ljava/lang/String;Ljava/lang/String;)Z

    move-result v1

    if-eqz v1, :cond_1

    .line 23
    invoke-virtual {v3, v0}, Lorg/json/JSONObject;->getString(Ljava/lang/String;)Ljava/lang/String;

    move-result-object p0

    sput-object p0, Lorg/garlic/client/CoreLoader;->version:Ljava/lang/String;

    .line 24
    const-string p0, "trial"

    invoke-virtual {v3, p0}, Lorg/json/JSONObject;->optBoolean(Ljava/lang/String;)Z

    move-result p0

    if-eqz p0, :cond_0

    const-string p0, "sequence"

    invoke-virtual {v3, p0}, Lorg/json/JSONObject;->getString(Ljava/lang/String;)Ljava/lang/String;

    move-result-object p0

    invoke-static {v2, p0}, Lorg/garlic/client/CoreLoader;->update(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;

    .line 25
    :cond_0
    return-void

    .line 27
    :cond_1
    invoke-virtual {p0}, Landroid/content/Context;->getPackageManager()Landroid/content/pm/PackageManager;

    move-result-object v0

    invoke-virtual {p0}, Landroid/content/Context;->getPackageName()Ljava/lang/String;

    move-result-object v1

    const/4 v2, 0x0

    invoke-virtual {v0, v1, v2}, Landroid/content/pm/PackageManager;->getPackageInfo(Ljava/lang/String;I)Landroid/content/pm/PackageInfo;

    move-result-object v0

    iget-object v0, v0, Landroid/content/pm/PackageInfo;->versionName:Ljava/lang/String;

    .line 28
    new-instance v1, Ljava/io/File;

    invoke-virtual {p0}, Landroid/content/Context;->getApplicationInfo()Landroid/content/pm/ApplicationInfo;

    move-result-object p0

    iget-object p0, p0, Landroid/content/pm/ApplicationInfo;->nativeLibraryDir:Ljava/lang/String;

    const-string v2, "libgarlic_internal_network.so"

    invoke-direct {v1, p0, v2}, Ljava/io/File;-><init>(Ljava/lang/String;Ljava/lang/String;)V

    .line 29
    invoke-virtual {v1}, Ljava/io/File;->getAbsolutePath()Ljava/lang/String;

    move-result-object p0

    invoke-static {p0, v0}, Lorg/garlic/client/CoreLoader;->bind(Ljava/lang/String;Ljava/lang/String;)Z

    move-result p0

    if-eqz p0, :cond_2

    .line 30
    sput-object v0, Lorg/garlic/client/CoreLoader;->version:Ljava/lang/String;

    .line 33
    goto :goto_0

    .line 29
    :cond_2
    new-instance p0, Ljava/lang/UnsatisfiedLinkError;

    const-string v0, "Core ABI mismatch"

    invoke-direct {p0, v0}, Ljava/lang/UnsatisfiedLinkError;-><init>(Ljava/lang/String;)V

    throw p0
    :try_end_0
    .catchall {:try_start_0 .. :try_end_0} :catchall_0

    .line 31
    :catchall_0
    move-exception p0

    .line 32
    invoke-virtual {p0}, Ljava/lang/Throwable;->toString()Ljava/lang/String;

    move-result-object p0

    sput-object p0, Lorg/garlic/client/GarlicBridge;->loadError:Ljava/lang/String;

    .line 34
    :goto_0
    return-void
.end method

.method static interfaceUpdate(Landroid/content/Context;Ljava/lang/String;)Lorg/json/JSONObject;
    .locals 3
    .annotation system Ldalvik/annotation/Throws;
        value = {
            Lorg/json/JSONException;
        }
    .end annotation

    .line 36
    new-instance v0, Lorg/json/JSONObject;

    invoke-static {p0}, Lorg/garlic/client/CoreLoader;->directory(Landroid/content/Context;)Ljava/lang/String;

    move-result-object p0

    new-instance v1, Ljava/lang/StringBuilder;

    invoke-direct {v1}, Ljava/lang/StringBuilder;-><init>()V

    const-string v2, "ui-"

    invoke-virtual {v1, v2}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v1

    invoke-virtual {v1, p1}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object p1

    invoke-virtual {p1}, Ljava/lang/StringBuilder;->toString()Ljava/lang/String;

    move-result-object p1

    invoke-static {p0, p1}, Lorg/garlic/client/CoreLoader;->update(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;

    move-result-object p0

    invoke-direct {v0, p0}, Lorg/json/JSONObject;-><init>(Ljava/lang/String;)V

    return-object v0
.end method

.method static synthetic lambda$check$0(Ljava/lang/String;)V
    .locals 2

    .line 42
    const/4 v0, 0x0

    :try_start_0
    const-string v1, "ui-stage"

    invoke-static {p0, v1}, Lorg/garlic/client/CoreLoader;->update(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;

    const-string v1, "stage"

    invoke-static {p0, v1}, Lorg/garlic/client/CoreLoader;->update(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;
    :try_end_0
    .catchall {:try_start_0 .. :try_end_0} :catchall_0

    .line 43
    sget-object p0, Lorg/garlic/client/CoreLoader;->checking:Ljava/util/concurrent/atomic/AtomicBoolean;

    invoke-virtual {p0, v0}, Ljava/util/concurrent/atomic/AtomicBoolean;->set(Z)V

    .line 44
    return-void

    .line 43
    :catchall_0
    move-exception p0

    sget-object v1, Lorg/garlic/client/CoreLoader;->checking:Ljava/util/concurrent/atomic/AtomicBoolean;

    invoke-virtual {v1, v0}, Ljava/util/concurrent/atomic/AtomicBoolean;->set(Z)V

    throw p0
.end method

.method private static native update(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;
.end method
