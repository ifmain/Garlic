.class Lorg/garlic/client/GarlicProxyService$1;
.super Ljava/lang/Object;
.source "GarlicProxyService.java"

# interfaces
.implements Ljava/lang/Runnable;


# annotations
.annotation system Ldalvik/annotation/EnclosingClass;
    value = Lorg/garlic/client/GarlicProxyService;
.end annotation

.annotation system Ldalvik/annotation/InnerClass;
    accessFlags = 0x0
    name = null
.end annotation


# instance fields
.field final synthetic this$0:Lorg/garlic/client/GarlicProxyService;


# direct methods
.method constructor <init>(Lorg/garlic/client/GarlicProxyService;)V
    .locals 0
    .annotation system Ldalvik/annotation/MethodParameters;
        accessFlags = {
            0x8010
        }
        names = {
            null
        }
    .end annotation

    .line 48
    iput-object p1, p0, Lorg/garlic/client/GarlicProxyService$1;->this$0:Lorg/garlic/client/GarlicProxyService;

    invoke-direct {p0}, Ljava/lang/Object;-><init>()V

    return-void
.end method


# virtual methods
.method public run()V
    .locals 26

    .line 49
    move-object/from16 v1, p0

    const-string v2, "peer_version"

    const-string v3, " \u00b7 "

    const-string v4, "via"

    const-string v5, "direct"

    sget-boolean v0, Lorg/garlic/client/GarlicProxyService;->isRunning:Z

    if-nez v0, :cond_0

    return-void

    .line 51
    :cond_0
    :try_start_0
    new-instance v6, Lorg/json/JSONObject;

    invoke-static {}, Lorg/garlic/client/GarlicBridge;->status()Ljava/lang/String;

    move-result-object v0

    invoke-direct {v6, v0}, Lorg/json/JSONObject;-><init>(Ljava/lang/String;)V

    .line 52
    const-string v0, "network_ready"

    const/4 v7, 0x0

    invoke-virtual {v6, v0, v7}, Lorg/json/JSONObject;->optBoolean(Ljava/lang/String;Z)Z

    move-result v0

    if-nez v0, :cond_1

    const/4 v0, 0x1

    goto :goto_0

    :cond_1
    move v0, v7

    :goto_0
    sput-boolean v0, Lorg/garlic/client/GarlicProxyService;->isConnecting:Z

    .line 53
    invoke-static {}, Landroid/os/SystemClock;->elapsedRealtime()J

    move-result-wide v8

    const-string v0, "rx_bytes"

    invoke-virtual {v6, v0}, Lorg/json/JSONObject;->optLong(Ljava/lang/String;)J

    move-result-wide v10

    const-string v0, "tx_bytes"

    invoke-virtual {v6, v0}, Lorg/json/JSONObject;->optLong(Ljava/lang/String;)J

    move-result-wide v12

    .line 54
    iget-object v0, v1, Lorg/garlic/client/GarlicProxyService$1;->this$0:Lorg/garlic/client/GarlicProxyService;

    invoke-static {v0}, Lorg/garlic/client/GarlicProxyService;->-$$Nest$fgetlastSample(Lorg/garlic/client/GarlicProxyService;)J

    move-result-wide v14

    move-wide/from16 v16, v8

    const-wide/16 v7, 0x0

    cmp-long v0, v14, v7

    if-lez v0, :cond_2

    iget-object v0, v1, Lorg/garlic/client/GarlicProxyService$1;->this$0:Lorg/garlic/client/GarlicProxyService;

    invoke-static {v0}, Lorg/garlic/client/GarlicProxyService;->-$$Nest$fgetlastSample(Lorg/garlic/client/GarlicProxyService;)J

    move-result-wide v14

    sub-long v14, v16, v14

    long-to-double v14, v14

    const-wide v18, 0x408f400000000000L    # 1000.0

    div-double v14, v14, v18

    const-wide v7, 0x3fb999999999999aL    # 0.1

    invoke-static {v7, v8, v14, v15}, Ljava/lang/Math;->max(DD)D

    move-result-wide v7

    invoke-static {}, Ljava/util/Locale;->getDefault()Ljava/util/Locale;

    move-result-object v0

    const-string v9, "\u2193 %.1f \u041a\u0411/\u0441 \u00b7 \u2191 %.1f \u041a\u0411/\u0441 \u00b7 %d \u043c\u0441"

    iget-object v14, v1, Lorg/garlic/client/GarlicProxyService$1;->this$0:Lorg/garlic/client/GarlicProxyService;

    invoke-static {v14}, Lorg/garlic/client/GarlicProxyService;->-$$Nest$fgetlastRx(Lorg/garlic/client/GarlicProxyService;)J

    move-result-wide v14

    sub-long v14, v10, v14

    move-wide/from16 v20, v7

    const-wide/16 v7, 0x0

    invoke-static {v7, v8, v14, v15}, Ljava/lang/Math;->max(JJ)J

    move-result-wide v14

    long-to-double v7, v14

    div-double v7, v7, v20

    const-wide/high16 v14, 0x4090000000000000L    # 1024.0

    div-double/2addr v7, v14

    invoke-static {v7, v8}, Ljava/lang/Double;->valueOf(D)Ljava/lang/Double;

    move-result-object v7

    iget-object v8, v1, Lorg/garlic/client/GarlicProxyService$1;->this$0:Lorg/garlic/client/GarlicProxyService;

    invoke-static {v8}, Lorg/garlic/client/GarlicProxyService;->-$$Nest$fgetlastTx(Lorg/garlic/client/GarlicProxyService;)J

    move-result-wide v22

    move-wide/from16 v24, v14

    sub-long v14, v12, v22

    move-object v8, v2

    move-object/from16 v22, v3

    const-wide/16 v2, 0x0

    invoke-static {v2, v3, v14, v15}, Ljava/lang/Math;->max(JJ)J

    move-result-wide v2

    long-to-double v2, v2

    div-double v2, v2, v20

    div-double v2, v2, v24

    invoke-static {v2, v3}, Ljava/lang/Double;->valueOf(D)Ljava/lang/Double;

    move-result-object v2

    const-string v3, "ping_ms"

    invoke-virtual {v6, v3}, Lorg/json/JSONObject;->optLong(Ljava/lang/String;)J

    move-result-wide v14

    invoke-static {v14, v15}, Ljava/lang/Long;->valueOf(J)Ljava/lang/Long;

    move-result-object v3

    filled-new-array {v7, v2, v3}, [Ljava/lang/Object;

    move-result-object v2

    invoke-static {v0, v9, v2}, Ljava/lang/String;->format(Ljava/util/Locale;Ljava/lang/String;[Ljava/lang/Object;)Ljava/lang/String;

    move-result-object v0

    sput-object v0, Lorg/garlic/client/GarlicProxyService;->statistics:Ljava/lang/String;

    goto :goto_1

    :cond_2
    move-object v8, v2

    move-object/from16 v22, v3

    .line 55
    :goto_1
    iget-object v0, v1, Lorg/garlic/client/GarlicProxyService$1;->this$0:Lorg/garlic/client/GarlicProxyService;

    move-wide/from16 v2, v16

    invoke-static {v0, v2, v3}, Lorg/garlic/client/GarlicProxyService;->-$$Nest$fputlastSample(Lorg/garlic/client/GarlicProxyService;J)V

    iget-object v0, v1, Lorg/garlic/client/GarlicProxyService$1;->this$0:Lorg/garlic/client/GarlicProxyService;

    invoke-static {v0, v10, v11}, Lorg/garlic/client/GarlicProxyService;->-$$Nest$fputlastRx(Lorg/garlic/client/GarlicProxyService;J)V

    iget-object v0, v1, Lorg/garlic/client/GarlicProxyService$1;->this$0:Lorg/garlic/client/GarlicProxyService;

    invoke-static {v0, v12, v13}, Lorg/garlic/client/GarlicProxyService;->-$$Nest$fputlastTx(Lorg/garlic/client/GarlicProxyService;J)V

    .line 56
    const-string v0, "last_route"

    invoke-virtual {v6, v0}, Lorg/json/JSONObject;->optJSONObject(Ljava/lang/String;)Lorg/json/JSONObject;

    move-result-object v2

    .line 57
    if-eqz v2, :cond_a

    const-string v0, "origin"

    invoke-virtual {v2, v0}, Lorg/json/JSONObject;->optString(Ljava/lang/String;)Ljava/lang/String;

    move-result-object v3
    :try_end_0
    .catchall {:try_start_0 .. :try_end_0} :catchall_0

    :try_start_1
    new-instance v0, Ljava/net/URI;

    invoke-direct {v0, v3}, Ljava/net/URI;-><init>(Ljava/lang/String;)V

    invoke-virtual {v0}, Ljava/net/URI;->getHost()Ljava/lang/String;

    move-result-object v3
    :try_end_1
    .catch Ljava/lang/Exception; {:try_start_1 .. :try_end_1} :catch_0
    .catchall {:try_start_1 .. :try_end_1} :catchall_0

    goto :goto_2

    :catch_0
    move-exception v0

    :goto_2
    :try_start_2
    invoke-virtual {v2, v4}, Lorg/json/JSONObject;->optString(Ljava/lang/String;)Ljava/lang/String;

    move-result-object v0

    invoke-virtual {v5, v0}, Ljava/lang/String;->equals(Ljava/lang/Object;)Z

    move-result v0

    if-eqz v0, :cond_3

    const-string v0, "\u041d\u0430\u043f\u0440\u044f\u043c\u0443\u044e"

    goto :goto_3

    :cond_3
    const-string v0, "\u0427\u0435\u0440\u0435\u0437 \u0420\u043e\u0441\u0441\u0438\u044e"

    :goto_3
    new-instance v7, Ljava/lang/StringBuilder;

    invoke-direct {v7}, Ljava/lang/StringBuilder;-><init>()V

    invoke-virtual {v7, v0}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v0

    move-object/from16 v7, v22

    invoke-virtual {v0, v7}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v0

    invoke-virtual {v0, v3}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v0

    invoke-virtual {v0}, Ljava/lang/StringBuilder;->toString()Ljava/lang/String;

    move-result-object v0

    .line 58
    const-string v3, "transport"

    invoke-virtual {v6, v3}, Lorg/json/JSONObject;->optJSONObject(Ljava/lang/String;)Lorg/json/JSONObject;

    move-result-object v3

    .line 59
    sget-object v9, Lorg/garlic/client/CoreLoader;->version:Ljava/lang/String;

    if-eqz v3, :cond_4

    const-string v10, "confirmed"

    invoke-virtual {v3, v10}, Lorg/json/JSONObject;->optBoolean(Ljava/lang/String;)Z

    move-result v10

    if-eqz v10, :cond_4

    const-string v10, "protocol"

    invoke-virtual {v3, v10}, Lorg/json/JSONObject;->optString(Ljava/lang/String;)Ljava/lang/String;

    move-result-object v10

    goto :goto_4

    :cond_4
    const-string v10, "\u043f\u0440\u043e\u0442\u043e\u043a\u043e\u043b \u043e\u043f\u0440\u0435\u0434\u0435\u043b\u044f\u0435\u0442\u0441\u044f"

    :goto_4
    new-instance v11, Ljava/lang/StringBuilder;

    invoke-direct {v11}, Ljava/lang/StringBuilder;-><init>()V

    invoke-virtual {v11, v0}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v0

    const-string v11, "\n\u042f\u0434\u0440\u043e "

    invoke-virtual {v0, v11}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v0

    invoke-virtual {v0, v9}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v0

    invoke-virtual {v0, v7}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v0

    invoke-virtual {v0, v10}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v0

    invoke-virtual {v0}, Ljava/lang/StringBuilder;->toString()Ljava/lang/String;

    move-result-object v0

    .line 60
    if-eqz v3, :cond_5

    const-string v7, "fallback_reason"

    invoke-virtual {v3, v7}, Lorg/json/JSONObject;->isNull(Ljava/lang/String;)Z

    move-result v7

    if-nez v7, :cond_5

    new-instance v7, Ljava/lang/StringBuilder;

    invoke-direct {v7}, Ljava/lang/StringBuilder;-><init>()V

    invoke-virtual {v7, v0}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v0

    const-string v7, " \u00b7 \u0441\u043e\u0432\u043c\u0435\u0441\u0442\u0438\u043c\u043e\u0441\u0442\u044c"

    invoke-virtual {v0, v7}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v0

    invoke-virtual {v0}, Ljava/lang/StringBuilder;->toString()Ljava/lang/String;

    move-result-object v0

    .line 61
    :cond_5
    if-eqz v3, :cond_6

    invoke-virtual {v3, v8}, Lorg/json/JSONObject;->isNull(Ljava/lang/String;)Z

    move-result v7

    if-nez v7, :cond_6

    invoke-virtual {v3, v8}, Lorg/json/JSONObject;->optString(Ljava/lang/String;)Ljava/lang/String;

    move-result-object v7

    new-instance v8, Ljava/lang/StringBuilder;

    invoke-direct {v8}, Ljava/lang/StringBuilder;-><init>()V

    invoke-virtual {v8, v0}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v0

    const-string v8, " \u00b7 \u0441\u0435\u0440\u0432\u0435\u0440 "

    invoke-virtual {v0, v8}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v0

    invoke-virtual {v0, v7}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v0

    invoke-virtual {v0}, Ljava/lang/StringBuilder;->toString()Ljava/lang/String;

    move-result-object v0

    .line 62
    :cond_6
    if-eqz v3, :cond_9

    invoke-virtual {v2, v4}, Lorg/json/JSONObject;->optString(Ljava/lang/String;)Ljava/lang/String;

    move-result-object v2

    invoke-virtual {v5, v2}, Ljava/lang/String;->equals(Ljava/lang/Object;)Z

    move-result v2

    if-nez v2, :cond_9

    .line 63
    const-string v2, "relay_protocol"

    const-string v4, ""

    invoke-virtual {v3, v2, v4}, Lorg/json/JSONObject;->optString(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;

    move-result-object v2

    .line 64
    const-string v3, "relay-shaped-v2"

    invoke-virtual {v3, v2}, Ljava/lang/String;->equals(Ljava/lang/Object;)Z

    move-result v3

    if-eqz v3, :cond_7

    const-string v2, "Relay v2"

    goto :goto_5

    :cond_7
    const-string v3, "relay-tls-v1"

    invoke-virtual {v3, v2}, Ljava/lang/String;->equals(Ljava/lang/Object;)Z

    move-result v2

    if-eqz v2, :cond_8

    const-string v2, "Relay v1 \u00b7 \u0441\u043e\u0432\u043c\u0435\u0441\u0442\u0438\u043c\u043e\u0441\u0442\u044c"

    goto :goto_5

    :cond_8
    const-string v2, "\u0432\u0435\u0440\u0441\u0438\u044f \u043d\u0435 \u0441\u043e\u043e\u0431\u0449\u0435\u043d\u0430"

    :goto_5
    new-instance v3, Ljava/lang/StringBuilder;

    invoke-direct {v3}, Ljava/lang/StringBuilder;-><init>()V

    invoke-virtual {v3, v0}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v0

    const-string v3, "\n\u041c\u0435\u0436\u0434\u0443 \u043d\u043e\u0434\u0430\u043c\u0438: "

    invoke-virtual {v0, v3}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v0

    invoke-virtual {v0, v2}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v0

    invoke-virtual {v0}, Ljava/lang/StringBuilder;->toString()Ljava/lang/String;

    move-result-object v0

    .line 66
    :cond_9
    invoke-static {v6}, Lorg/garlic/client/GarlicProxyService;->channelSummary(Lorg/json/JSONObject;)Ljava/lang/String;

    move-result-object v2

    new-instance v3, Ljava/lang/StringBuilder;

    invoke-direct {v3}, Ljava/lang/StringBuilder;-><init>()V

    invoke-virtual {v3, v0}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v0

    const-string v3, "\n\u041a\u0430\u043d\u0430\u043b\u044b: "

    invoke-virtual {v0, v3}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v0

    invoke-virtual {v0, v2}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v0

    invoke-virtual {v0}, Ljava/lang/StringBuilder;->toString()Ljava/lang/String;

    move-result-object v0

    sput-object v0, Lorg/garlic/client/GarlicProxyService;->currentRoute:Ljava/lang/String;

    .line 68
    :cond_a
    const-string v0, "proxy"

    invoke-virtual {v6, v0}, Lorg/json/JSONObject;->optBoolean(Ljava/lang/String;)Z

    move-result v0

    if-eqz v0, :cond_c

    const-string v0, "tunnel"

    invoke-virtual {v6, v0}, Lorg/json/JSONObject;->optBoolean(Ljava/lang/String;)Z

    move-result v0
    :try_end_2
    .catchall {:try_start_2 .. :try_end_2} :catchall_0

    if-nez v0, :cond_b

    goto :goto_6

    .line 72
    :cond_b
    nop

    .line 73
    iget-object v0, v1, Lorg/garlic/client/GarlicProxyService$1;->this$0:Lorg/garlic/client/GarlicProxyService;

    iget-object v2, v1, Lorg/garlic/client/GarlicProxyService$1;->this$0:Lorg/garlic/client/GarlicProxyService;

    const-string v3, "garlic_prefs"

    const/4 v4, 0x0

    invoke-virtual {v2, v3, v4}, Lorg/garlic/client/GarlicProxyService;->getSharedPreferences(Ljava/lang/String;I)Landroid/content/SharedPreferences;

    move-result-object v2

    const-string v3, "mode"

    const-string v4, "auto"

    invoke-interface {v2, v3, v4}, Landroid/content/SharedPreferences;->getString(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;

    move-result-object v2

    const-string v3, "127.0.0.1:1080"

    invoke-static {v0, v2, v3}, Lorg/garlic/client/GarlicProxyService;->-$$Nest$mstartOngoingForeground(Lorg/garlic/client/GarlicProxyService;Ljava/lang/String;Ljava/lang/String;)V

    .line 74
    iget-object v0, v1, Lorg/garlic/client/GarlicProxyService$1;->this$0:Lorg/garlic/client/GarlicProxyService;

    invoke-static {v0}, Lorg/garlic/client/GarlicProxyService;->-$$Nest$fgetmonitor(Lorg/garlic/client/GarlicProxyService;)Landroid/os/Handler;

    move-result-object v0

    const-wide/16 v2, 0x3e8

    invoke-virtual {v0, v1, v2, v3}, Landroid/os/Handler;->postDelayed(Ljava/lang/Runnable;J)Z

    .line 75
    return-void

    .line 69
    :cond_c
    :goto_6
    :try_start_3
    const-string v0, "\u041f\u043e\u0434\u043a\u043b\u044e\u0447\u0435\u043d\u0438\u0435 \u043f\u0440\u0435\u0440\u0432\u0430\u043d\u043e. \u041f\u0440\u043e\u0432\u0435\u0440\u044c\u0442\u0435 \u0441\u0435\u0442\u044c \u0438\u043b\u0438 \u043f\u0435\u0440\u0435\u043a\u043b\u044e\u0447\u0438\u0442\u0435 \u0434\u043e\u0441\u0442\u0443\u043f \u043d\u0430 \u044d\u0442\u043e \u0443\u0441\u0442\u0440\u043e\u0439\u0441\u0442\u0432\u043e"

    sput-object v0, Lorg/garlic/client/GarlicProxyService;->lastError:Ljava/lang/String;

    .line 70
    iget-object v0, v1, Lorg/garlic/client/GarlicProxyService$1;->this$0:Lorg/garlic/client/GarlicProxyService;

    invoke-static {v0}, Lorg/garlic/client/GarlicProxyService;->-$$Nest$mretryConnection(Lorg/garlic/client/GarlicProxyService;)V
    :try_end_3
    .catchall {:try_start_3 .. :try_end_3} :catchall_0

    return-void

    .line 72
    :catchall_0
    move-exception v0

    const-string v0, "\u0421\u0435\u0442\u0435\u0432\u043e\u0439 \u043c\u043e\u0434\u0443\u043b\u044c \u043e\u0441\u0442\u0430\u043d\u043e\u0432\u043b\u0435\u043d"

    sput-object v0, Lorg/garlic/client/GarlicProxyService;->lastError:Ljava/lang/String;

    iget-object v0, v1, Lorg/garlic/client/GarlicProxyService$1;->this$0:Lorg/garlic/client/GarlicProxyService;

    invoke-static {v0}, Lorg/garlic/client/GarlicProxyService;->-$$Nest$mretryConnection(Lorg/garlic/client/GarlicProxyService;)V

    return-void
.end method
