.class public Lorg/garlic/client/GarlicProxyService;
.super Landroid/net/VpnService;
.source "GarlicProxyService.java"


# static fields
.field public static final CHANNEL_ID:Ljava/lang/String; = "garlic_proxy_ongoing"

.field public static final NOTIFICATION_ID:I = 0x3e9

.field public static volatile currentRoute:Ljava/lang/String;

.field public static volatile isConnecting:Z

.field public static volatile isRunning:Z

.field public static volatile lastError:Ljava/lang/String;

.field private static final lifecycle:Ljava/util/concurrent/ExecutorService;

.field public static volatile statistics:Ljava/lang/String;


# instance fields
.field private cleanupQueued:Z

.field private final connectionState:Lorg/garlic/client/ConnectionState;

.field private final health:Ljava/lang/Runnable;

.field private lastRx:J

.field private lastSample:J

.field private lastTx:J

.field private final monitor:Landroid/os/Handler;

.field private networkCallback:Landroid/net/ConnectivityManager$NetworkCallback;

.field private final reconnect:Ljava/lang/Runnable;

.field private retrySeconds:I

.field private volatile starting:Z

.field private tunnelInterface:Landroid/os/ParcelFileDescriptor;

.field private volatile wanted:Z


# direct methods
.method static bridge synthetic -$$Nest$fgetlastRx(Lorg/garlic/client/GarlicProxyService;)J
    .locals 2

    iget-wide v0, p0, Lorg/garlic/client/GarlicProxyService;->lastRx:J

    return-wide v0
.end method

.method static bridge synthetic -$$Nest$fgetlastSample(Lorg/garlic/client/GarlicProxyService;)J
    .locals 2

    iget-wide v0, p0, Lorg/garlic/client/GarlicProxyService;->lastSample:J

    return-wide v0
.end method

.method static bridge synthetic -$$Nest$fgetlastTx(Lorg/garlic/client/GarlicProxyService;)J
    .locals 2

    iget-wide v0, p0, Lorg/garlic/client/GarlicProxyService;->lastTx:J

    return-wide v0
.end method

.method static bridge synthetic -$$Nest$fgetmonitor(Lorg/garlic/client/GarlicProxyService;)Landroid/os/Handler;
    .locals 0

    iget-object p0, p0, Lorg/garlic/client/GarlicProxyService;->monitor:Landroid/os/Handler;

    return-object p0
.end method

.method static bridge synthetic -$$Nest$fputlastRx(Lorg/garlic/client/GarlicProxyService;J)V
    .locals 0

    iput-wide p1, p0, Lorg/garlic/client/GarlicProxyService;->lastRx:J

    return-void
.end method

.method static bridge synthetic -$$Nest$fputlastSample(Lorg/garlic/client/GarlicProxyService;J)V
    .locals 0

    iput-wide p1, p0, Lorg/garlic/client/GarlicProxyService;->lastSample:J

    return-void
.end method

.method static bridge synthetic -$$Nest$fputlastTx(Lorg/garlic/client/GarlicProxyService;J)V
    .locals 0

    iput-wide p1, p0, Lorg/garlic/client/GarlicProxyService;->lastTx:J

    return-void
.end method

.method static bridge synthetic -$$Nest$mretryConnection(Lorg/garlic/client/GarlicProxyService;)V
    .locals 0

    invoke-direct {p0}, Lorg/garlic/client/GarlicProxyService;->retryConnection()V

    return-void
.end method

.method static bridge synthetic -$$Nest$mstartOngoingForeground(Lorg/garlic/client/GarlicProxyService;Ljava/lang/String;Ljava/lang/String;)V
    .locals 0

    invoke-direct {p0, p1, p2}, Lorg/garlic/client/GarlicProxyService;->startOngoingForeground(Ljava/lang/String;Ljava/lang/String;)V

    return-void
.end method

.method static constructor <clinit>()V
    .locals 1

    .line 28
    const/4 v0, 0x0

    sput-boolean v0, Lorg/garlic/client/GarlicProxyService;->isRunning:Z

    .line 29
    sput-boolean v0, Lorg/garlic/client/GarlicProxyService;->isConnecting:Z

    .line 30
    const/4 v0, 0x0

    sput-object v0, Lorg/garlic/client/GarlicProxyService;->lastError:Ljava/lang/String;

    .line 31
    const-string v0, ""

    sput-object v0, Lorg/garlic/client/GarlicProxyService;->currentRoute:Ljava/lang/String;

    .line 32
    sput-object v0, Lorg/garlic/client/GarlicProxyService;->statistics:Ljava/lang/String;

    .line 90
    invoke-static {}, Ljava/util/concurrent/Executors;->newSingleThreadExecutor()Ljava/util/concurrent/ExecutorService;

    move-result-object v0

    sput-object v0, Lorg/garlic/client/GarlicProxyService;->lifecycle:Ljava/util/concurrent/ExecutorService;

    return-void
.end method

.method public constructor <init>()V
    .locals 3

    .line 25
    invoke-direct {p0}, Landroid/net/VpnService;-><init>()V

    .line 33
    const-wide/16 v0, 0x0

    iput-wide v0, p0, Lorg/garlic/client/GarlicProxyService;->lastRx:J

    iput-wide v0, p0, Lorg/garlic/client/GarlicProxyService;->lastTx:J

    iput-wide v0, p0, Lorg/garlic/client/GarlicProxyService;->lastSample:J

    .line 34
    const/4 v0, 0x0

    iput-boolean v0, p0, Lorg/garlic/client/GarlicProxyService;->wanted:Z

    .line 35
    const/4 v1, 0x2

    iput v1, p0, Lorg/garlic/client/GarlicProxyService;->retrySeconds:I

    .line 36
    new-instance v1, Lorg/garlic/client/GarlicProxyService$5;

    invoke-direct {v1, p0}, Lorg/garlic/client/GarlicProxyService$5;-><init>(Lorg/garlic/client/GarlicProxyService;)V

    iput-object v1, p0, Lorg/garlic/client/GarlicProxyService;->reconnect:Ljava/lang/Runnable;

    .line 47
    new-instance v1, Landroid/os/Handler;

    invoke-static {}, Landroid/os/Looper;->getMainLooper()Landroid/os/Looper;

    move-result-object v2

    invoke-direct {v1, v2}, Landroid/os/Handler;-><init>(Landroid/os/Looper;)V

    iput-object v1, p0, Lorg/garlic/client/GarlicProxyService;->monitor:Landroid/os/Handler;

    .line 48
    new-instance v1, Lorg/garlic/client/GarlicProxyService$1;

    invoke-direct {v1, p0}, Lorg/garlic/client/GarlicProxyService$1;-><init>(Lorg/garlic/client/GarlicProxyService;)V

    iput-object v1, p0, Lorg/garlic/client/GarlicProxyService;->health:Ljava/lang/Runnable;

    .line 91
    iput-boolean v0, p0, Lorg/garlic/client/GarlicProxyService;->starting:Z

    .line 92
    new-instance v1, Lorg/garlic/client/ConnectionState;

    invoke-direct {v1}, Lorg/garlic/client/ConnectionState;-><init>()V

    iput-object v1, p0, Lorg/garlic/client/GarlicProxyService;->connectionState:Lorg/garlic/client/ConnectionState;

    .line 93
    iput-boolean v0, p0, Lorg/garlic/client/GarlicProxyService;->cleanupQueued:Z

    return-void
.end method

.method static channelSummary(Lorg/json/JSONObject;)Ljava/lang/String;
    .locals 8

    .line 78
    const-string v0, "transport_modes"

    invoke-virtual {p0, v0}, Lorg/json/JSONObject;->optJSONObject(Ljava/lang/String;)Lorg/json/JSONObject;

    move-result-object v0

    .line 79
    if-eqz v0, :cond_2

    .line 80
    new-instance v1, Ljava/util/TreeSet;

    invoke-direct {v1}, Ljava/util/TreeSet;-><init>()V

    .line 81
    invoke-virtual {v0}, Lorg/json/JSONObject;->keys()Ljava/util/Iterator;

    move-result-object v2

    :goto_0
    invoke-interface {v2}, Ljava/util/Iterator;->hasNext()Z

    move-result v3

    if-eqz v3, :cond_1

    .line 82
    invoke-interface {v2}, Ljava/util/Iterator;->next()Ljava/lang/Object;

    move-result-object v3

    check-cast v3, Ljava/lang/String;

    .line 83
    invoke-virtual {v0, v3}, Lorg/json/JSONObject;->optLong(Ljava/lang/String;)J

    move-result-wide v4

    const-wide/16 v6, 0x0

    cmp-long v4, v4, v6

    if-lez v4, :cond_0

    invoke-interface {v1, v3}, Ljava/util/SortedSet;->add(Ljava/lang/Object;)Z

    .line 84
    :cond_0
    goto :goto_0

    .line 85
    :cond_1
    invoke-interface {v1}, Ljava/util/SortedSet;->isEmpty()Z

    move-result v2

    if-nez v2, :cond_2

    const-string p0, " + "

    invoke-static {p0, v1}, Landroid/text/TextUtils;->join(Ljava/lang/CharSequence;Ljava/lang/Iterable;)Ljava/lang/String;

    move-result-object p0

    return-object p0

    .line 87
    :cond_2
    const-string v1, "network_ready"

    invoke-virtual {p0, v1}, Lorg/json/JSONObject;->optBoolean(Ljava/lang/String;)Z

    move-result p0

    if-nez p0, :cond_3

    const-string p0, "\u043e\u0436\u0438\u0434\u0430\u043d\u0438\u0435 \u0441\u043e\u0435\u0434\u0438\u043d\u0435\u043d\u0438\u044f"

    return-object p0

    .line 88
    :cond_3
    if-nez v0, :cond_4

    const-string p0, "\u043d\u0435\u0442 \u0434\u0430\u043d\u043d\u044b\u0445"

    goto :goto_1

    :cond_4
    const-string p0, "\u0441\u0435\u0439\u0447\u0430\u0441 \u043d\u0435\u0442 \u0430\u043a\u0442\u0438\u0432\u043d\u044b\u0445 \u043a\u0430\u043d\u0430\u043b\u043e\u0432"

    :goto_1
    return-object p0
.end method

.method private cleanupNative()V
    .locals 3

    .line 291
    const-string v0, "connectivity"

    invoke-virtual {p0, v0}, Lorg/garlic/client/GarlicProxyService;->getSystemService(Ljava/lang/String;)Ljava/lang/Object;

    move-result-object v0

    check-cast v0, Landroid/net/ConnectivityManager;

    .line 292
    const/4 v1, 0x0

    if-eqz v0, :cond_0

    iget-object v2, p0, Lorg/garlic/client/GarlicProxyService;->networkCallback:Landroid/net/ConnectivityManager$NetworkCallback;

    if-eqz v2, :cond_0

    .line 294
    :try_start_0
    iget-object v2, p0, Lorg/garlic/client/GarlicProxyService;->networkCallback:Landroid/net/ConnectivityManager$NetworkCallback;

    invoke-virtual {v0, v2}, Landroid/net/ConnectivityManager;->unregisterNetworkCallback(Landroid/net/ConnectivityManager$NetworkCallback;)V
    :try_end_0
    .catchall {:try_start_0 .. :try_end_0} :catchall_0

    .line 296
    goto :goto_0

    .line 295
    :catchall_0
    move-exception v0

    .line 297
    :goto_0
    iput-object v1, p0, Lorg/garlic/client/GarlicProxyService;->networkCallback:Landroid/net/ConnectivityManager$NetworkCallback;

    .line 300
    :cond_0
    :try_start_1
    sget-object v0, Lorg/garlic/client/GarlicBridge;->loadError:Ljava/lang/String;

    if-nez v0, :cond_1

    .line 301
    invoke-static {}, Lorg/garlic/client/GarlicBridge;->stopProxy()Ljava/lang/String;
    :try_end_1
    .catchall {:try_start_1 .. :try_end_1} :catchall_1

    .line 304
    :cond_1
    goto :goto_1

    .line 303
    :catchall_1
    move-exception v0

    .line 305
    :goto_1
    iget-object v0, p0, Lorg/garlic/client/GarlicProxyService;->tunnelInterface:Landroid/os/ParcelFileDescriptor;

    if-eqz v0, :cond_2

    .line 307
    :try_start_2
    iget-object v0, p0, Lorg/garlic/client/GarlicProxyService;->tunnelInterface:Landroid/os/ParcelFileDescriptor;

    invoke-virtual {v0}, Landroid/os/ParcelFileDescriptor;->close()V
    :try_end_2
    .catchall {:try_start_2 .. :try_end_2} :catchall_2

    .line 309
    goto :goto_2

    .line 308
    :catchall_2
    move-exception v0

    .line 310
    :goto_2
    iput-object v1, p0, Lorg/garlic/client/GarlicProxyService;->tunnelInterface:Landroid/os/ParcelFileDescriptor;

    .line 312
    :cond_2
    return-void
.end method

.method private establishHttpProxyConfig()V
    .locals 4

    .line 255
    :try_start_0
    iget-object v0, p0, Lorg/garlic/client/GarlicProxyService;->tunnelInterface:Landroid/os/ParcelFileDescriptor;

    if-eqz v0, :cond_0

    .line 256
    return-void

    .line 258
    :cond_0
    new-instance v0, Landroid/net/VpnService$Builder;

    invoke-direct {v0, p0}, Landroid/net/VpnService$Builder;-><init>(Landroid/net/VpnService;)V

    const-string v1, "Garlic"

    .line 259
    invoke-virtual {v0, v1}, Landroid/net/VpnService$Builder;->setSession(Ljava/lang/String;)Landroid/net/VpnService$Builder;

    move-result-object v0

    .line 260
    const/16 v1, 0x5dc

    invoke-virtual {v0, v1}, Landroid/net/VpnService$Builder;->setMtu(I)Landroid/net/VpnService$Builder;

    move-result-object v0

    const-string v1, "10.254.254.1"

    .line 261
    const/16 v2, 0x20

    invoke-virtual {v0, v1, v2}, Landroid/net/VpnService$Builder;->addAddress(Ljava/lang/String;I)Landroid/net/VpnService$Builder;

    move-result-object v0

    const-string v1, "0.0.0.0"

    .line 262
    const/4 v2, 0x0

    invoke-virtual {v0, v1, v2}, Landroid/net/VpnService$Builder;->addRoute(Ljava/lang/String;I)Landroid/net/VpnService$Builder;

    move-result-object v0

    const-string v1, "1.1.1.1"

    .line 265
    invoke-virtual {v0, v1}, Landroid/net/VpnService$Builder;->addDnsServer(Ljava/lang/String;)Landroid/net/VpnService$Builder;

    move-result-object v0

    .line 266
    invoke-virtual {p0}, Lorg/garlic/client/GarlicProxyService;->getPackageName()Ljava/lang/String;

    move-result-object v1

    invoke-virtual {v0, v1}, Landroid/net/VpnService$Builder;->addDisallowedApplication(Ljava/lang/String;)Landroid/net/VpnService$Builder;

    move-result-object v0

    .line 267
    const-string v1, "garlic_prefs"

    invoke-virtual {p0, v1, v2}, Lorg/garlic/client/GarlicProxyService;->getSharedPreferences(Ljava/lang/String;I)Landroid/content/SharedPreferences;

    move-result-object v1

    const-string v2, "bypass_packages"

    invoke-static {}, Ljava/util/Collections;->emptySet()Ljava/util/Set;

    move-result-object v3

    invoke-interface {v1, v2, v3}, Landroid/content/SharedPreferences;->getStringSet(Ljava/lang/String;Ljava/util/Set;)Ljava/util/Set;

    move-result-object v1

    invoke-interface {v1}, Ljava/util/Set;->iterator()Ljava/util/Iterator;

    move-result-object v1

    :goto_0
    invoke-interface {v1}, Ljava/util/Iterator;->hasNext()Z

    move-result v2

    if-eqz v2, :cond_2

    invoke-interface {v1}, Ljava/util/Iterator;->next()Ljava/lang/Object;

    move-result-object v2

    check-cast v2, Ljava/lang/String;

    .line 268
    invoke-virtual {p0}, Lorg/garlic/client/GarlicProxyService;->getPackageName()Ljava/lang/String;

    move-result-object v3

    invoke-virtual {v2, v3}, Ljava/lang/String;->equals(Ljava/lang/Object;)Z

    move-result v3
    :try_end_0
    .catch Ljava/lang/Exception; {:try_start_0 .. :try_end_0} :catch_1

    if-nez v3, :cond_1

    :try_start_1
    invoke-virtual {v0, v2}, Landroid/net/VpnService$Builder;->addDisallowedApplication(Ljava/lang/String;)Landroid/net/VpnService$Builder;
    :try_end_1
    .catch Landroid/content/pm/PackageManager$NameNotFoundException; {:try_start_1 .. :try_end_1} :catch_0
    .catch Ljava/lang/Exception; {:try_start_1 .. :try_end_1} :catch_1

    goto :goto_1

    :catch_0
    move-exception v2

    .line 269
    :cond_1
    :goto_1
    goto :goto_0

    .line 270
    :cond_2
    :try_start_2
    invoke-virtual {v0}, Landroid/net/VpnService$Builder;->establish()Landroid/os/ParcelFileDescriptor;

    move-result-object v0

    iput-object v0, p0, Lorg/garlic/client/GarlicProxyService;->tunnelInterface:Landroid/os/ParcelFileDescriptor;

    .line 271
    iget-object v0, p0, Lorg/garlic/client/GarlicProxyService;->tunnelInterface:Landroid/os/ParcelFileDescriptor;

    if-eqz v0, :cond_3

    .line 276
    nop

    .line 277
    return-void

    .line 272
    :cond_3
    new-instance v0, Ljava/lang/IllegalStateException;

    const-string v1, "VPN permission is required"

    invoke-direct {v0, v1}, Ljava/lang/IllegalStateException;-><init>(Ljava/lang/String;)V

    throw v0
    :try_end_2
    .catch Ljava/lang/Exception; {:try_start_2 .. :try_end_2} :catch_1

    .line 274
    :catch_1
    move-exception v0

    .line 275
    new-instance v1, Ljava/lang/IllegalStateException;

    const-string v2, "VPN interface setup failed"

    invoke-direct {v1, v2, v0}, Ljava/lang/IllegalStateException;-><init>(Ljava/lang/String;Ljava/lang/Throwable;)V

    throw v1
.end method

.method private retryConnection()V
    .locals 6

    .line 38
    iget-boolean v0, p0, Lorg/garlic/client/GarlicProxyService;->wanted:Z

    if-nez v0, :cond_0

    return-void

    .line 39
    :cond_0
    invoke-direct {p0}, Lorg/garlic/client/GarlicProxyService;->shutdownProxy()V

    .line 40
    const/4 v0, 0x1

    sput-boolean v0, Lorg/garlic/client/GarlicProxyService;->isConnecting:Z

    const-string v0, "\u0412\u043e\u0441\u0441\u0442\u0430\u043d\u0430\u0432\u043b\u0438\u0432\u0430\u0435\u043c \u043f\u043e\u0434\u043a\u043b\u044e\u0447\u0435\u043d\u0438\u0435\u2026"

    sput-object v0, Lorg/garlic/client/GarlicProxyService;->currentRoute:Ljava/lang/String;

    .line 41
    const-string v0, "garlic_prefs"

    const/4 v1, 0x0

    invoke-virtual {p0, v0, v1}, Lorg/garlic/client/GarlicProxyService;->getSharedPreferences(Ljava/lang/String;I)Landroid/content/SharedPreferences;

    move-result-object v0

    const-string v1, "mode"

    const-string v2, "auto"

    invoke-interface {v0, v1, v2}, Landroid/content/SharedPreferences;->getString(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;

    move-result-object v0

    const-string v1, "127.0.0.1:1080"

    invoke-direct {p0, v0, v1}, Lorg/garlic/client/GarlicProxyService;->startOngoingForeground(Ljava/lang/String;Ljava/lang/String;)V

    .line 42
    iget-object v0, p0, Lorg/garlic/client/GarlicProxyService;->monitor:Landroid/os/Handler;

    iget-object v1, p0, Lorg/garlic/client/GarlicProxyService;->reconnect:Ljava/lang/Runnable;

    invoke-virtual {v0, v1}, Landroid/os/Handler;->removeCallbacks(Ljava/lang/Runnable;)V

    .line 43
    iget-object v0, p0, Lorg/garlic/client/GarlicProxyService;->monitor:Landroid/os/Handler;

    iget-object v1, p0, Lorg/garlic/client/GarlicProxyService;->reconnect:Ljava/lang/Runnable;

    iget v2, p0, Lorg/garlic/client/GarlicProxyService;->retrySeconds:I

    int-to-long v2, v2

    const-wide/16 v4, 0x3e8

    mul-long/2addr v2, v4

    invoke-virtual {v0, v1, v2, v3}, Landroid/os/Handler;->postDelayed(Ljava/lang/Runnable;J)Z

    .line 44
    iget v0, p0, Lorg/garlic/client/GarlicProxyService;->retrySeconds:I

    mul-int/lit8 v0, v0, 0x2

    const/16 v1, 0x1e

    invoke-static {v1, v0}, Ljava/lang/Math;->min(II)I

    move-result v0

    iput v0, p0, Lorg/garlic/client/GarlicProxyService;->retrySeconds:I

    .line 45
    return-void
.end method

.method private declared-synchronized shutdownProxy()V
    .locals 2

    monitor-enter p0

    .line 280
    const/4 v0, 0x0

    :try_start_0
    sput-boolean v0, Lorg/garlic/client/GarlicProxyService;->isRunning:Z

    sput-boolean v0, Lorg/garlic/client/GarlicProxyService;->isConnecting:Z

    .line 281
    const-string v1, ""

    sput-object v1, Lorg/garlic/client/GarlicProxyService;->currentRoute:Ljava/lang/String;

    .line 282
    iget-object v1, p0, Lorg/garlic/client/GarlicProxyService;->connectionState:Lorg/garlic/client/ConnectionState;

    invoke-virtual {v1}, Lorg/garlic/client/ConnectionState;->invalidate()V

    .line 283
    iput-boolean v0, p0, Lorg/garlic/client/GarlicProxyService;->starting:Z

    .line 284
    iget-object v0, p0, Lorg/garlic/client/GarlicProxyService;->monitor:Landroid/os/Handler;

    iget-object v1, p0, Lorg/garlic/client/GarlicProxyService;->health:Ljava/lang/Runnable;

    invoke-virtual {v0, v1}, Landroid/os/Handler;->removeCallbacks(Ljava/lang/Runnable;)V

    .line 285
    iget-boolean v0, p0, Lorg/garlic/client/GarlicProxyService;->cleanupQueued:Z
    :try_end_0
    .catchall {:try_start_0 .. :try_end_0} :catchall_0

    if-eqz v0, :cond_0

    monitor-exit p0

    return-void

    .line 286
    :cond_0
    const/4 v0, 0x1

    :try_start_1
    iput-boolean v0, p0, Lorg/garlic/client/GarlicProxyService;->cleanupQueued:Z

    .line 287
    sget-object v0, Lorg/garlic/client/GarlicProxyService;->lifecycle:Ljava/util/concurrent/ExecutorService;

    new-instance v1, Lorg/garlic/client/GarlicProxyService$0;

    invoke-direct {v1, p0}, Lorg/garlic/client/GarlicProxyService$0;-><init>(Lorg/garlic/client/GarlicProxyService;)V

    invoke-interface {v0, v1}, Ljava/util/concurrent/ExecutorService;->execute(Ljava/lang/Runnable;)V
    :try_end_1
    .catchall {:try_start_1 .. :try_end_1} :catchall_0

    .line 288
    monitor-exit p0

    return-void

    .line 279
    :catchall_0
    move-exception v0

    :try_start_2
    monitor-exit p0
    :try_end_2
    .catchall {:try_start_2 .. :try_end_2} :catchall_0

    throw v0
.end method

.method private startOngoingForeground(Ljava/lang/String;Ljava/lang/String;)V
    .locals 9

    .line 179
    :try_start_0
    const-string p2, "notification"

    invoke-virtual {p0, p2}, Lorg/garlic/client/GarlicProxyService;->getSystemService(Ljava/lang/String;)Ljava/lang/Object;

    move-result-object p2

    check-cast p2, Landroid/app/NotificationManager;
    :try_end_0
    .catchall {:try_start_0 .. :try_end_0} :catchall_1

    .line 180
    const-string v0, "garlic_proxy_ongoing"

    const/4 v1, 0x0

    if-eqz p2, :cond_0

    .line 181
    :try_start_1
    new-instance v2, Landroid/app/NotificationChannel;

    const-string v3, "Garlic"

    const/4 v4, 0x2

    invoke-direct {v2, v0, v3, v4}, Landroid/app/NotificationChannel;-><init>(Ljava/lang/String;Ljava/lang/CharSequence;I)V

    .line 186
    const-string v3, "\u0421\u0442\u0430\u0442\u0443\u0441 \u043f\u043e\u0434\u043a\u043b\u044e\u0447\u0435\u043d\u0438\u044f, \u043c\u0430\u0440\u0448\u0440\u0443\u0442 \u0438 \u0441\u043a\u043e\u0440\u043e\u0441\u0442\u044c"

    invoke-virtual {v2, v3}, Landroid/app/NotificationChannel;->setDescription(Ljava/lang/String;)V

    .line 187
    invoke-virtual {v2, v1}, Landroid/app/NotificationChannel;->setShowBadge(Z)V

    .line 188
    invoke-virtual {p2, v2}, Landroid/app/NotificationManager;->createNotificationChannel(Landroid/app/NotificationChannel;)V

    .line 191
    :cond_0
    new-instance v2, Landroid/content/Intent;

    const-class v3, Lorg/garlic/client/MainActivity;

    invoke-direct {v2, p0, v3}, Landroid/content/Intent;-><init>(Landroid/content/Context;Ljava/lang/Class;)V

    .line 192
    const/high16 v3, 0x24000000

    invoke-virtual {v2, v3}, Landroid/content/Intent;->setFlags(I)Landroid/content/Intent;

    .line 193
    const/high16 v3, 0xc000000

    invoke-static {p0, v1, v2, v3}, Landroid/app/PendingIntent;->getActivity(Landroid/content/Context;ILandroid/content/Intent;I)Landroid/app/PendingIntent;

    move-result-object v2

    .line 200
    new-instance v4, Landroid/content/Intent;

    const-class v5, Lorg/garlic/client/GarlicProxyService;

    invoke-direct {v4, p0, v5}, Landroid/content/Intent;-><init>(Landroid/content/Context;Ljava/lang/Class;)V

    .line 201
    const-string v5, "STOP"

    invoke-virtual {v4, v5}, Landroid/content/Intent;->setAction(Ljava/lang/String;)Landroid/content/Intent;

    .line 202
    const/4 v5, 0x1

    invoke-static {p0, v5, v4, v3}, Landroid/app/PendingIntent;->getService(Landroid/content/Context;ILandroid/content/Intent;I)Landroid/app/PendingIntent;

    move-result-object v3

    .line 209
    const-string v4, "\u0410\u0432\u0442\u043e\u043c\u0430\u0442\u0438\u0447\u0435\u0441\u043a\u0438\u0439 \u043c\u0430\u0440\u0448\u0440\u0443\u0442"

    .line 210
    const-string v6, "direct"

    invoke-virtual {v6, p1}, Ljava/lang/String;->equals(Ljava/lang/Object;)Z

    move-result v6

    if-eqz v6, :cond_1

    .line 211
    const-string v4, "\u041d\u0430\u043f\u0440\u044f\u043c\u0443\u044e"

    goto :goto_0

    .line 212
    :cond_1
    const-string v6, "ru"

    invoke-virtual {v6, p1}, Ljava/lang/String;->equals(Ljava/lang/Object;)Z

    move-result p1

    if-eqz p1, :cond_2

    .line 213
    const-string v4, "\u0427\u0435\u0440\u0435\u0437 \u0420\u043e\u0441\u0441\u0438\u044e"

    .line 216
    :cond_2
    :goto_0
    new-instance p1, Landroid/app/Notification$Builder;

    invoke-direct {p1, p0, v0}, Landroid/app/Notification$Builder;-><init>(Landroid/content/Context;Ljava/lang/String;)V

    .line 217
    invoke-virtual {p0}, Lorg/garlic/client/GarlicProxyService;->getResources()Landroid/content/res/Resources;

    move-result-object v0

    const-string v6, "ic_garlic_notification"

    const-string v7, "drawable"

    invoke-virtual {p0}, Lorg/garlic/client/GarlicProxyService;->getPackageName()Ljava/lang/String;

    move-result-object v8

    invoke-virtual {v0, v6, v7, v8}, Landroid/content/res/Resources;->getIdentifier(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)I

    move-result v0

    invoke-virtual {p1, v0}, Landroid/app/Notification$Builder;->setSmallIcon(I)Landroid/app/Notification$Builder;

    move-result-object p1

    .line 218
    sget-boolean v0, Lorg/garlic/client/GarlicProxyService;->isRunning:Z

    if-eqz v0, :cond_3

    const-string v0, "Garlic \u00b7 \u041f\u043e\u0434\u043a\u043b\u044e\u0447\u0435\u043d\u043e"

    goto :goto_1

    :cond_3
    const-string v0, "Garlic \u00b7 \u041f\u043e\u0434\u043a\u043b\u044e\u0447\u0435\u043d\u0438\u0435\u2026"

    :goto_1
    invoke-virtual {p1, v0}, Landroid/app/Notification$Builder;->setContentTitle(Ljava/lang/CharSequence;)Landroid/app/Notification$Builder;

    move-result-object p1

    .line 219
    sget-object v0, Lorg/garlic/client/GarlicProxyService;->currentRoute:Ljava/lang/String;

    invoke-virtual {v0}, Ljava/lang/String;->isEmpty()Z

    move-result v0

    if-eqz v0, :cond_4

    goto :goto_2

    :cond_4
    sget-object v4, Lorg/garlic/client/GarlicProxyService;->currentRoute:Ljava/lang/String;

    :goto_2
    invoke-virtual {p1, v4}, Landroid/app/Notification$Builder;->setContentText(Ljava/lang/CharSequence;)Landroid/app/Notification$Builder;

    move-result-object p1

    .line 220
    const-string v0, "garlic_prefs"

    invoke-virtual {p0, v0, v1}, Lorg/garlic/client/GarlicProxyService;->getSharedPreferences(Ljava/lang/String;I)Landroid/content/SharedPreferences;

    move-result-object v0

    const-string v1, "show_statistics"

    invoke-interface {v0, v1, v5}, Landroid/content/SharedPreferences;->getBoolean(Ljava/lang/String;Z)Z

    move-result v0

    if-eqz v0, :cond_5

    sget-object v0, Lorg/garlic/client/GarlicProxyService;->statistics:Ljava/lang/String;

    goto :goto_3

    :cond_5
    const-string v0, ""

    :goto_3
    invoke-virtual {p1, v0}, Landroid/app/Notification$Builder;->setSubText(Ljava/lang/CharSequence;)Landroid/app/Notification$Builder;

    move-result-object p1

    .line 221
    invoke-virtual {p1, v5}, Landroid/app/Notification$Builder;->setOngoing(Z)Landroid/app/Notification$Builder;

    move-result-object p1

    .line 222
    invoke-virtual {p1, v5}, Landroid/app/Notification$Builder;->setOnlyAlertOnce(Z)Landroid/app/Notification$Builder;

    move-result-object p1

    .line 223
    invoke-virtual {p1, v2}, Landroid/app/Notification$Builder;->setContentIntent(Landroid/app/PendingIntent;)Landroid/app/Notification$Builder;

    move-result-object p1

    new-instance v0, Landroid/app/Notification$Action$Builder;

    const-string v1, "\u041e\u0442\u043a\u043b\u044e\u0447\u0438\u0442\u044c"

    const v2, 0x1080038

    invoke-direct {v0, v2, v1, v3}, Landroid/app/Notification$Action$Builder;-><init>(ILjava/lang/CharSequence;Landroid/app/PendingIntent;)V

    .line 228
    invoke-virtual {v0}, Landroid/app/Notification$Action$Builder;->build()Landroid/app/Notification$Action;

    move-result-object v0

    .line 224
    invoke-virtual {p1, v0}, Landroid/app/Notification$Builder;->addAction(Landroid/app/Notification$Action;)Landroid/app/Notification$Builder;

    move-result-object p1

    .line 230
    sget v0, Landroid/os/Build$VERSION;->SDK_INT:I

    const/16 v1, 0x1f

    if-lt v0, v1, :cond_6

    .line 231
    invoke-virtual {p1, v5}, Landroid/app/Notification$Builder;->setForegroundServiceBehavior(I)Landroid/app/Notification$Builder;

    .line 234
    :cond_6
    invoke-virtual {p1}, Landroid/app/Notification$Builder;->build()Landroid/app/Notification;

    move-result-object p1
    :try_end_1
    .catchall {:try_start_1 .. :try_end_1} :catchall_1

    .line 236
    const/16 v0, 0x3e9

    :try_start_2
    sget-boolean v1, Lorg/garlic/client/GarlicProxyService;->isRunning:Z

    if-eqz v1, :cond_7

    if-eqz p2, :cond_7

    invoke-virtual {p2, v0, p1}, Landroid/app/NotificationManager;->notify(ILandroid/app/Notification;)V

    return-void

    .line 237
    :cond_7
    sget p2, Landroid/os/Build$VERSION;->SDK_INT:I

    const/16 v1, 0x22

    if-lt p2, v1, :cond_8

    .line 238
    const/high16 p2, 0x40000000    # 2.0f

    invoke-virtual {p0, v0, p1, p2}, Lorg/garlic/client/GarlicProxyService;->startForeground(ILandroid/app/Notification;I)V

    goto :goto_4

    .line 244
    :cond_8
    invoke-virtual {p0, v0, p1}, Lorg/garlic/client/GarlicProxyService;->startForeground(ILandroid/app/Notification;)V
    :try_end_2
    .catchall {:try_start_2 .. :try_end_2} :catchall_0

    .line 248
    :goto_4
    goto :goto_5

    .line 246
    :catchall_0
    move-exception p2

    .line 247
    :try_start_3
    invoke-virtual {p0, v0, p1}, Lorg/garlic/client/GarlicProxyService;->startForeground(ILandroid/app/Notification;)V
    :try_end_3
    .catchall {:try_start_3 .. :try_end_3} :catchall_1

    .line 250
    :goto_5
    goto :goto_6

    .line 249
    :catchall_1
    move-exception p1

    .line 251
    :goto_6
    return-void
.end method


# virtual methods
.method synthetic lambda$new$0$org-garlic-client-GarlicProxyService()V
    .locals 2

    .line 36
    iget-boolean v0, p0, Lorg/garlic/client/GarlicProxyService;->wanted:Z

    if-eqz v0, :cond_0

    new-instance v0, Landroid/content/Intent;

    const-class v1, Lorg/garlic/client/GarlicProxyService;

    invoke-direct {v0, p0, v1}, Landroid/content/Intent;-><init>(Landroid/content/Context;Ljava/lang/Class;)V

    const/4 v1, 0x0

    invoke-virtual {p0, v0, v1, v1}, Lorg/garlic/client/GarlicProxyService;->onStartCommand(Landroid/content/Intent;II)I

    :cond_0
    return-void
.end method

.method synthetic lambda$onStartCommand$1$org-garlic-client-GarlicProxyService(J)V
    .locals 2

    .line 157
    iget-object v0, p0, Lorg/garlic/client/GarlicProxyService;->connectionState:Lorg/garlic/client/ConnectionState;

    invoke-virtual {v0, p1, p2}, Lorg/garlic/client/ConnectionState;->current(J)Z

    move-result p1

    if-nez p1, :cond_0

    return-void

    .line 158
    :cond_0
    const/4 p1, 0x1

    sput-boolean p1, Lorg/garlic/client/GarlicProxyService;->isRunning:Z

    sput-boolean p1, Lorg/garlic/client/GarlicProxyService;->isConnecting:Z

    const/4 p1, 0x0

    iput-boolean p1, p0, Lorg/garlic/client/GarlicProxyService;->starting:Z

    const/4 p1, 0x2

    iput p1, p0, Lorg/garlic/client/GarlicProxyService;->retrySeconds:I

    const/4 p1, 0x0

    sput-object p1, Lorg/garlic/client/GarlicProxyService;->lastError:Ljava/lang/String;

    .line 159
    iget-object p1, p0, Lorg/garlic/client/GarlicProxyService;->monitor:Landroid/os/Handler;

    iget-object p2, p0, Lorg/garlic/client/GarlicProxyService;->health:Ljava/lang/Runnable;

    invoke-virtual {p1, p2}, Landroid/os/Handler;->removeCallbacks(Ljava/lang/Runnable;)V

    iget-object p1, p0, Lorg/garlic/client/GarlicProxyService;->monitor:Landroid/os/Handler;

    iget-object p2, p0, Lorg/garlic/client/GarlicProxyService;->health:Ljava/lang/Runnable;

    const-wide/16 v0, 0x3e8

    invoke-virtual {p1, p2, v0, v1}, Landroid/os/Handler;->postDelayed(Ljava/lang/Runnable;J)Z

    .line 160
    return-void
.end method

.method synthetic lambda$onStartCommand$2$org-garlic-client-GarlicProxyService(JLjava/lang/Throwable;)V
    .locals 1

    .line 165
    iget-object v0, p0, Lorg/garlic/client/GarlicProxyService;->connectionState:Lorg/garlic/client/ConnectionState;

    invoke-virtual {v0, p1, p2}, Lorg/garlic/client/ConnectionState;->current(J)Z

    move-result p1

    if-nez p1, :cond_0

    return-void

    .line 166
    :cond_0
    const/4 p1, 0x0

    iput-boolean p1, p0, Lorg/garlic/client/GarlicProxyService;->starting:Z

    invoke-virtual {p3}, Ljava/lang/Throwable;->getMessage()Ljava/lang/String;

    move-result-object p2

    sput-object p2, Lorg/garlic/client/GarlicProxyService;->lastError:Ljava/lang/String;

    .line 167
    sget-object p2, Lorg/garlic/client/GarlicProxyService;->lastError:Ljava/lang/String;

    invoke-static {p2}, Lorg/garlic/client/ConnectionState;->retryAllowed(Ljava/lang/String;)Z

    move-result p2

    if-eqz p2, :cond_1

    invoke-direct {p0}, Lorg/garlic/client/GarlicProxyService;->retryConnection()V

    goto :goto_0

    .line 168
    :cond_1
    iput-boolean p1, p0, Lorg/garlic/client/GarlicProxyService;->wanted:Z

    iget-object p1, p0, Lorg/garlic/client/GarlicProxyService;->connectionState:Lorg/garlic/client/ConnectionState;

    invoke-virtual {p1}, Lorg/garlic/client/ConnectionState;->stop()V

    invoke-direct {p0}, Lorg/garlic/client/GarlicProxyService;->shutdownProxy()V

    const/4 p1, 0x1

    invoke-virtual {p0, p1}, Lorg/garlic/client/GarlicProxyService;->stopForeground(Z)V

    invoke-virtual {p0}, Lorg/garlic/client/GarlicProxyService;->stopSelf()V

    .line 169
    :goto_0
    return-void
.end method

.method synthetic lambda$onStartCommand$3$org-garlic-client-GarlicProxyService(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;J)V
    .locals 4

    .line 136
    const-string v0, "ok"

    :try_start_0
    new-instance v1, Ljava/io/File;

    invoke-virtual {p0}, Lorg/garlic/client/GarlicProxyService;->getFilesDir()Ljava/io/File;

    move-result-object v2

    const-string v3, "garlic"

    invoke-direct {v1, v2, v3}, Ljava/io/File;-><init>(Ljava/io/File;Ljava/lang/String;)V

    .line 137
    invoke-virtual {v1}, Ljava/io/File;->exists()Z

    move-result v2

    if-nez v2, :cond_0

    .line 138
    invoke-virtual {v1}, Ljava/io/File;->mkdirs()Z

    .line 140
    :cond_0
    sget-object v2, Lorg/garlic/client/GarlicBridge;->loadError:Ljava/lang/String;

    if-nez v2, :cond_6

    .line 141
    sget-object v2, Lorg/garlic/client/GarlicBridge;->loadError:Ljava/lang/String;

    if-nez v2, :cond_5

    .line 142
    if-eqz p1, :cond_1

    invoke-virtual {p1}, Ljava/lang/String;->trim()Ljava/lang/String;

    move-result-object v2

    invoke-virtual {v2}, Ljava/lang/String;->isEmpty()Z

    move-result v2

    if-nez v2, :cond_1

    .line 143
    new-instance v2, Ljava/lang/StringBuilder;

    invoke-direct {v2}, Ljava/lang/StringBuilder;-><init>()V

    invoke-virtual {v2, p2}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v2

    const-string v3, ","

    invoke-virtual {v2, v3}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v2

    invoke-virtual {v2, p3}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v2

    invoke-virtual {v2}, Ljava/lang/StringBuilder;->toString()Ljava/lang/String;

    move-result-object v2

    .line 144
    invoke-virtual {v1}, Ljava/io/File;->getAbsolutePath()Ljava/lang/String;

    move-result-object v3

    invoke-virtual {p1}, Ljava/lang/String;->trim()Ljava/lang/String;

    move-result-object p1

    invoke-static {v3, v2, p1}, Lorg/garlic/client/GarlicBridge;->saveKey(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;

    .line 146
    :cond_1
    invoke-virtual {v1}, Ljava/io/File;->getAbsolutePath()Ljava/lang/String;

    move-result-object p1

    invoke-static {p1, p4, p2, p3, p5}, Lorg/garlic/client/GarlicBridge;->startProxy(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;

    move-result-object p1

    .line 147
    new-instance p2, Lorg/json/JSONObject;

    invoke-direct {p2, p1}, Lorg/json/JSONObject;-><init>(Ljava/lang/String;)V

    const/4 p3, 0x0

    invoke-virtual {p2, v0, p3}, Lorg/json/JSONObject;->optBoolean(Ljava/lang/String;Z)Z

    move-result p2

    if-eqz p2, :cond_4

    .line 150
    iget-object p1, p0, Lorg/garlic/client/GarlicProxyService;->connectionState:Lorg/garlic/client/ConnectionState;

    invoke-virtual {p1, p6, p7}, Lorg/garlic/client/ConnectionState;->current(J)Z

    move-result p1

    if-nez p1, :cond_2

    return-void

    .line 151
    :cond_2
    invoke-direct {p0}, Lorg/garlic/client/GarlicProxyService;->establishHttpProxyConfig()V

    .line 152
    iget-object p1, p0, Lorg/garlic/client/GarlicProxyService;->tunnelInterface:Landroid/os/ParcelFileDescriptor;

    invoke-virtual {p1}, Landroid/os/ParcelFileDescriptor;->getFd()I

    move-result p1

    invoke-static {p1}, Lorg/garlic/client/GarlicBridge;->startTun(I)Ljava/lang/String;

    move-result-object p1

    .line 153
    new-instance p2, Lorg/json/JSONObject;

    invoke-direct {p2, p1}, Lorg/json/JSONObject;-><init>(Ljava/lang/String;)V

    invoke-virtual {p2, v0, p3}, Lorg/json/JSONObject;->optBoolean(Ljava/lang/String;Z)Z

    move-result p1

    if-eqz p1, :cond_3

    .line 156
    iget-object p1, p0, Lorg/garlic/client/GarlicProxyService;->monitor:Landroid/os/Handler;

    new-instance p2, Lorg/garlic/client/GarlicProxyService$2;

    invoke-direct {p2, p0, p6, p7}, Lorg/garlic/client/GarlicProxyService$2;-><init>(Lorg/garlic/client/GarlicProxyService;J)V

    invoke-virtual {p1, p2}, Landroid/os/Handler;->post(Ljava/lang/Runnable;)Z

    goto :goto_0

    .line 154
    :cond_3
    new-instance p1, Ljava/lang/IllegalStateException;

    const-string p2, "Device tunnel startup failed"

    invoke-direct {p1, p2}, Ljava/lang/IllegalStateException;-><init>(Ljava/lang/String;)V

    throw p1

    .line 148
    :cond_4
    new-instance p2, Ljava/lang/IllegalStateException;

    new-instance p3, Lorg/json/JSONObject;

    invoke-direct {p3, p1}, Lorg/json/JSONObject;-><init>(Ljava/lang/String;)V

    const-string p1, "error"

    const-string p4, "\u041d\u0435 \u0443\u0434\u0430\u043b\u043e\u0441\u044c \u0437\u0430\u043f\u0443\u0441\u0442\u0438\u0442\u044c \u043f\u043e\u0434\u043a\u043b\u044e\u0447\u0435\u043d\u0438\u0435"

    invoke-virtual {p3, p1, p4}, Lorg/json/JSONObject;->optString(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;

    move-result-object p1

    invoke-direct {p2, p1}, Ljava/lang/IllegalStateException;-><init>(Ljava/lang/String;)V

    throw p2

    .line 171
    :cond_5
    :goto_0
    nop

    .line 173
    return-void

    .line 140
    :cond_6
    new-instance p1, Ljava/lang/IllegalStateException;

    sget-object p2, Lorg/garlic/client/GarlicBridge;->loadError:Ljava/lang/String;

    new-instance p3, Ljava/lang/StringBuilder;

    invoke-direct {p3}, Ljava/lang/StringBuilder;-><init>()V

    const-string p4, "\u0421\u0435\u0442\u0435\u0432\u043e\u0439 \u043c\u043e\u0434\u0443\u043b\u044c \u043d\u0435\u0434\u043e\u0441\u0442\u0443\u043f\u0435\u043d: "

    invoke-virtual {p3, p4}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object p3

    invoke-virtual {p3, p2}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object p2

    invoke-virtual {p2}, Ljava/lang/StringBuilder;->toString()Ljava/lang/String;

    move-result-object p2

    invoke-direct {p1, p2}, Ljava/lang/IllegalStateException;-><init>(Ljava/lang/String;)V

    throw p1
    :try_end_0
    .catchall {:try_start_0 .. :try_end_0} :catchall_0

    .line 162
    :catchall_0
    move-exception p1

    .line 163
    const-string p2, "Garlic"

    const-string p3, "Proxy service startup failed"

    invoke-static {p2, p3, p1}, Landroid/util/Log;->e(Ljava/lang/String;Ljava/lang/String;Ljava/lang/Throwable;)I

    .line 164
    iget-object p2, p0, Lorg/garlic/client/GarlicProxyService;->monitor:Landroid/os/Handler;

    new-instance p3, Lorg/garlic/client/GarlicProxyService$3;

    invoke-direct {p3, p0, p6, p7, p1}, Lorg/garlic/client/GarlicProxyService$3;-><init>(Lorg/garlic/client/GarlicProxyService;JLjava/lang/Throwable;)V

    invoke-virtual {p2, p3}, Landroid/os/Handler;->post(Ljava/lang/Runnable;)Z

    .line 170
    return-void
.end method

.method synthetic lambda$shutdownProxy$4$org-garlic-client-GarlicProxyService()V
    .locals 0

    .line 287
    invoke-direct {p0}, Lorg/garlic/client/GarlicProxyService;->cleanupNative()V

    return-void
.end method

.method public onDestroy()V
    .locals 2

    .line 325
    const/4 v0, 0x0

    iput-boolean v0, p0, Lorg/garlic/client/GarlicProxyService;->wanted:Z

    iget-object v0, p0, Lorg/garlic/client/GarlicProxyService;->connectionState:Lorg/garlic/client/ConnectionState;

    invoke-virtual {v0}, Lorg/garlic/client/ConnectionState;->stop()V

    iget-object v0, p0, Lorg/garlic/client/GarlicProxyService;->monitor:Landroid/os/Handler;

    iget-object v1, p0, Lorg/garlic/client/GarlicProxyService;->reconnect:Ljava/lang/Runnable;

    invoke-virtual {v0, v1}, Landroid/os/Handler;->removeCallbacks(Ljava/lang/Runnable;)V

    .line 326
    invoke-direct {p0}, Lorg/garlic/client/GarlicProxyService;->shutdownProxy()V

    .line 327
    invoke-super {p0}, Landroid/net/VpnService;->onDestroy()V

    .line 328
    return-void
.end method

.method public onRevoke()V
    .locals 2

    .line 316
    const/4 v0, 0x0

    iput-boolean v0, p0, Lorg/garlic/client/GarlicProxyService;->wanted:Z

    iget-object v0, p0, Lorg/garlic/client/GarlicProxyService;->connectionState:Lorg/garlic/client/ConnectionState;

    invoke-virtual {v0}, Lorg/garlic/client/ConnectionState;->stop()V

    iget-object v0, p0, Lorg/garlic/client/GarlicProxyService;->monitor:Landroid/os/Handler;

    iget-object v1, p0, Lorg/garlic/client/GarlicProxyService;->reconnect:Ljava/lang/Runnable;

    invoke-virtual {v0, v1}, Landroid/os/Handler;->removeCallbacks(Ljava/lang/Runnable;)V

    .line 317
    invoke-direct {p0}, Lorg/garlic/client/GarlicProxyService;->shutdownProxy()V

    .line 318
    const/4 v0, 0x1

    invoke-virtual {p0, v0}, Lorg/garlic/client/GarlicProxyService;->stopForeground(Z)V

    .line 319
    invoke-virtual {p0}, Lorg/garlic/client/GarlicProxyService;->stopSelf()V

    .line 320
    invoke-super {p0}, Landroid/net/VpnService;->onRevoke()V

    .line 321
    return-void
.end method

.method public onStartCommand(Landroid/content/Intent;II)I
    .locals 10

    .line 99
    const/4 p2, 0x0

    const/4 v0, 0x1

    if-eqz p1, :cond_0

    const-string v1, "STOP"

    invoke-virtual {p1}, Landroid/content/Intent;->getAction()Ljava/lang/String;

    move-result-object v2

    invoke-virtual {v1, v2}, Ljava/lang/String;->equals(Ljava/lang/Object;)Z

    move-result v1

    if-eqz v1, :cond_0

    .line 100
    iput-boolean p2, p0, Lorg/garlic/client/GarlicProxyService;->wanted:Z

    iget-object p1, p0, Lorg/garlic/client/GarlicProxyService;->connectionState:Lorg/garlic/client/ConnectionState;

    invoke-virtual {p1}, Lorg/garlic/client/ConnectionState;->stop()V

    iget-object p1, p0, Lorg/garlic/client/GarlicProxyService;->monitor:Landroid/os/Handler;

    iget-object p2, p0, Lorg/garlic/client/GarlicProxyService;->reconnect:Ljava/lang/Runnable;

    invoke-virtual {p1, p2}, Landroid/os/Handler;->removeCallbacks(Ljava/lang/Runnable;)V

    .line 101
    invoke-direct {p0}, Lorg/garlic/client/GarlicProxyService;->shutdownProxy()V

    .line 103
    :try_start_0
    invoke-virtual {p0, v0}, Lorg/garlic/client/GarlicProxyService;->stopForeground(Z)V
    :try_end_0
    .catchall {:try_start_0 .. :try_end_0} :catchall_0

    .line 105
    goto :goto_0

    .line 104
    :catchall_0
    move-exception v0

    .line 106
    :goto_0
    invoke-virtual {p0, p3}, Lorg/garlic/client/GarlicProxyService;->stopSelf(I)V

    .line 107
    const/4 p1, 0x2

    return p1

    .line 110
    :cond_0
    iput-boolean v0, p0, Lorg/garlic/client/GarlicProxyService;->wanted:Z

    .line 111
    iget-object p3, p0, Lorg/garlic/client/GarlicProxyService;->connectionState:Lorg/garlic/client/ConnectionState;

    invoke-virtual {p3}, Lorg/garlic/client/ConnectionState;->begin()J

    move-result-wide v8

    .line 112
    const-wide/16 v1, 0x0

    cmp-long p3, v8, v1

    if-gez p3, :cond_1

    return v0

    .line 113
    :cond_1
    const/4 p3, 0x0

    sput-object p3, Lorg/garlic/client/GarlicProxyService;->lastError:Ljava/lang/String;

    .line 114
    iput-boolean p2, p0, Lorg/garlic/client/GarlicProxyService;->cleanupQueued:Z

    .line 115
    iput-boolean v0, p0, Lorg/garlic/client/GarlicProxyService;->starting:Z

    sput-boolean v0, Lorg/garlic/client/GarlicProxyService;->isConnecting:Z

    .line 116
    const-string p3, "garlic_prefs"

    invoke-virtual {p0, p3, p2}, Lorg/garlic/client/GarlicProxyService;->getSharedPreferences(Ljava/lang/String;I)Landroid/content/SharedPreferences;

    move-result-object p2

    .line 117
    const-string p3, "mode"

    if-eqz p1, :cond_2

    invoke-virtual {p1, p3}, Landroid/content/Intent;->getStringExtra(Ljava/lang/String;)Ljava/lang/String;

    move-result-object v1

    if-eqz v1, :cond_2

    .line 118
    invoke-virtual {p1, p3}, Landroid/content/Intent;->getStringExtra(Ljava/lang/String;)Ljava/lang/String;

    move-result-object p3

    move-object v6, p3

    goto :goto_1

    .line 119
    :cond_2
    const-string v1, "auto"

    invoke-interface {p2, p3, v1}, Landroid/content/SharedPreferences;->getString(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;

    move-result-object p3

    move-object v6, p3

    .line 120
    :goto_1
    const-string p3, "direct_urls"

    if-eqz p1, :cond_3

    invoke-virtual {p1, p3}, Landroid/content/Intent;->getStringExtra(Ljava/lang/String;)Ljava/lang/String;

    move-result-object v1

    if-eqz v1, :cond_3

    .line 121
    invoke-virtual {p1, p3}, Landroid/content/Intent;->getStringExtra(Ljava/lang/String;)Ljava/lang/String;

    move-result-object p3

    move-object v4, p3

    goto :goto_2

    .line 122
    :cond_3
    const-string v1, "https://212.34.147.227:8443"

    invoke-interface {p2, p3, v1}, Landroid/content/SharedPreferences;->getString(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;

    move-result-object p3

    move-object v4, p3

    .line 123
    :goto_2
    const-string p3, "ru_urls"

    if-eqz p1, :cond_4

    invoke-virtual {p1, p3}, Landroid/content/Intent;->getStringExtra(Ljava/lang/String;)Ljava/lang/String;

    move-result-object v1

    if-eqz v1, :cond_4

    .line 124
    invoke-virtual {p1, p3}, Landroid/content/Intent;->getStringExtra(Ljava/lang/String;)Ljava/lang/String;

    move-result-object p3

    move-object v5, p3

    goto :goto_3

    .line 125
    :cond_4
    const-string v1, "https://194.87.111.169:8443"

    invoke-interface {p2, p3, v1}, Landroid/content/SharedPreferences;->getString(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;

    move-result-object p3

    move-object v5, p3

    .line 126
    :goto_3
    const-string p3, "key"

    if-eqz p1, :cond_5

    invoke-virtual {p1, p3}, Landroid/content/Intent;->getStringExtra(Ljava/lang/String;)Ljava/lang/String;

    move-result-object v1

    if-eqz v1, :cond_5

    .line 127
    invoke-virtual {p1, p3}, Landroid/content/Intent;->getStringExtra(Ljava/lang/String;)Ljava/lang/String;

    move-result-object p1

    move-object v3, p1

    goto :goto_4

    .line 128
    :cond_5
    const-string p1, ""

    invoke-interface {p2, p3, p1}, Landroid/content/SharedPreferences;->getString(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;

    move-result-object p1

    move-object v3, p1

    .line 129
    :goto_4
    nop

    .line 131
    const-string v7, "127.0.0.1:1080"

    invoke-direct {p0, v6, v7}, Lorg/garlic/client/GarlicProxyService;->startOngoingForeground(Ljava/lang/String;Ljava/lang/String;)V

    .line 133
    sget-object p1, Lorg/garlic/client/GarlicProxyService;->lifecycle:Ljava/util/concurrent/ExecutorService;

    new-instance v1, Lorg/garlic/client/GarlicProxyService$4;

    move-object v2, p0

    invoke-direct/range {v1 .. v9}, Lorg/garlic/client/GarlicProxyService$4;-><init>(Lorg/garlic/client/GarlicProxyService;Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;J)V

    invoke-interface {p1, v1}, Ljava/util/concurrent/ExecutorService;->execute(Ljava/lang/Runnable;)V

    .line 174
    return v0
.end method
