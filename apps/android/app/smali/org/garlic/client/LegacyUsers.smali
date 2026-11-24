.class final Lorg/garlic/client/LegacyUsers;
.super Ljava/lang/Object;
.source "LegacyUsers.java"


# instance fields
.field private final active:Ljava/util/function/BooleanSupplier;

.field private final dir:Ljava/lang/String;

.field private directory:Lorg/json/JSONObject;

.field private final executor:Ljava/util/concurrent/Executor;

.field private origin:Ljava/lang/String;

.field private final origins:Ljava/lang/String;

.field private final ui:Landroid/app/Activity;


# direct methods
.method constructor <init>(Landroid/app/Activity;Ljava/util/concurrent/Executor;Ljava/lang/String;Ljava/lang/String;Ljava/util/function/BooleanSupplier;)V
    .locals 0

    .line 19
    invoke-direct {p0}, Ljava/lang/Object;-><init>()V

    .line 20
    iput-object p1, p0, Lorg/garlic/client/LegacyUsers;->ui:Landroid/app/Activity;

    iput-object p2, p0, Lorg/garlic/client/LegacyUsers;->executor:Ljava/util/concurrent/Executor;

    iput-object p3, p0, Lorg/garlic/client/LegacyUsers;->dir:Ljava/lang/String;

    iput-object p4, p0, Lorg/garlic/client/LegacyUsers;->origins:Ljava/lang/String;

    iput-object p5, p0, Lorg/garlic/client/LegacyUsers;->active:Ljava/util/function/BooleanSupplier;

    .line 21
    return-void
.end method

.method private create()V
    .locals 11

    .line 93
    new-instance v0, Landroid/widget/LinearLayout;

    iget-object v1, p0, Lorg/garlic/client/LegacyUsers;->ui:Landroid/app/Activity;

    invoke-direct {v0, v1}, Landroid/widget/LinearLayout;-><init>(Landroid/content/Context;)V

    const/4 v1, 0x1

    invoke-virtual {v0, v1}, Landroid/widget/LinearLayout;->setOrientation(I)V

    .line 94
    new-instance v5, Landroid/widget/EditText;

    iget-object v2, p0, Lorg/garlic/client/LegacyUsers;->ui:Landroid/app/Activity;

    invoke-direct {v5, v2}, Landroid/widget/EditText;-><init>(Landroid/content/Context;)V

    const-string v2, "\u0418\u043c\u044f \u043f\u043e\u043b\u044c\u0437\u043e\u0432\u0430\u0442\u0435\u043b\u044f"

    invoke-virtual {v5, v2}, Landroid/widget/EditText;->setHint(Ljava/lang/CharSequence;)V

    invoke-virtual {v0, v5}, Landroid/widget/LinearLayout;->addView(Landroid/view/View;)V

    .line 95
    invoke-static {}, Ljava/util/Calendar;->getInstance()Ljava/util/Calendar;

    move-result-object v6

    const/4 v2, 0x2

    invoke-virtual {v6, v2, v1}, Ljava/util/Calendar;->add(II)V

    .line 96
    new-instance v1, Landroid/widget/Button;

    iget-object v2, p0, Lorg/garlic/client/LegacyUsers;->ui:Landroid/app/Activity;

    invoke-direct {v1, v2}, Landroid/widget/Button;-><init>(Landroid/content/Context;)V

    new-instance v2, Lorg/garlic/client/LegacyUsers$8;

    invoke-direct {v2, v1, v6}, Lorg/garlic/client/LegacyUsers$8;-><init>(Landroid/widget/Button;Ljava/util/Calendar;)V

    .line 97
    invoke-interface {v2}, Ljava/lang/Runnable;->run()V

    new-instance v3, Lorg/garlic/client/LegacyUsers$9;

    invoke-direct {v3, p0, v6, v2}, Lorg/garlic/client/LegacyUsers$9;-><init>(Lorg/garlic/client/LegacyUsers;Ljava/util/Calendar;Ljava/lang/Runnable;)V

    invoke-virtual {v1, v3}, Landroid/widget/Button;->setOnClickListener(Landroid/view/View$OnClickListener;)V

    .line 98
    invoke-virtual {v0, v1}, Landroid/widget/LinearLayout;->addView(Landroid/view/View;)V

    .line 99
    new-instance v7, Landroid/widget/CheckBox;

    iget-object v1, p0, Lorg/garlic/client/LegacyUsers;->ui:Landroid/app/Activity;

    invoke-direct {v7, v1}, Landroid/widget/CheckBox;-><init>(Landroid/content/Context;)V

    const-string v1, "\u0411\u0435\u0437 \u0441\u0440\u043e\u043a\u0430"

    invoke-virtual {v7, v1}, Landroid/widget/CheckBox;->setText(Ljava/lang/CharSequence;)V

    invoke-virtual {v0, v7}, Landroid/widget/LinearLayout;->addView(Landroid/view/View;)V

    .line 100
    new-instance v4, Ljava/util/LinkedHashMap;

    invoke-direct {v4}, Ljava/util/LinkedHashMap;-><init>()V

    iget-object v1, p0, Lorg/garlic/client/LegacyUsers;->directory:Lorg/json/JSONObject;

    const-string v2, "routes"

    invoke-virtual {v1, v2}, Lorg/json/JSONObject;->optJSONArray(Ljava/lang/String;)Lorg/json/JSONArray;

    move-result-object v1

    .line 101
    const/4 v2, 0x0

    :goto_0
    invoke-virtual {v1}, Lorg/json/JSONArray;->length()I

    move-result v3

    if-ge v2, v3, :cond_2

    invoke-virtual {v1, v2}, Lorg/json/JSONArray;->optJSONObject(I)Lorg/json/JSONObject;

    move-result-object v3

    .line 102
    const-string v8, "enable"

    invoke-virtual {v3, v8}, Lorg/json/JSONObject;->optBoolean(Ljava/lang/String;)Z

    move-result v8

    if-eqz v8, :cond_1

    const-string v8, "nodeId"

    invoke-virtual {v3, v8}, Lorg/json/JSONObject;->optInt(Ljava/lang/String;)I

    move-result v8

    iget-object v9, p0, Lorg/garlic/client/LegacyUsers;->directory:Lorg/json/JSONObject;

    const-string v10, "worker"

    invoke-virtual {v9, v10}, Lorg/json/JSONObject;->optInt(Ljava/lang/String;)I

    move-result v9

    if-ne v8, v9, :cond_1

    const-string v8, "vmess"

    const-string v9, "trojan"

    const-string v10, "vless"

    filled-new-array {v10, v8, v9}, [Ljava/lang/String;

    move-result-object v8

    invoke-static {v8}, Ljava/util/Arrays;->asList([Ljava/lang/Object;)Ljava/util/List;

    move-result-object v8

    const-string v9, "protocol"

    invoke-virtual {v3, v9}, Lorg/json/JSONObject;->optString(Ljava/lang/String;)Ljava/lang/String;

    move-result-object v9

    invoke-interface {v8, v9}, Ljava/util/List;->contains(Ljava/lang/Object;)Z

    move-result v8

    if-nez v8, :cond_0

    goto :goto_1

    .line 103
    :cond_0
    new-instance v8, Landroid/widget/CheckBox;

    iget-object v9, p0, Lorg/garlic/client/LegacyUsers;->ui:Landroid/app/Activity;

    invoke-direct {v8, v9}, Landroid/widget/CheckBox;-><init>(Landroid/content/Context;)V

    invoke-direct {p0, v3}, Lorg/garlic/client/LegacyUsers;->route(Lorg/json/JSONObject;)Ljava/lang/String;

    move-result-object v9

    invoke-virtual {v8, v9}, Landroid/widget/CheckBox;->setText(Ljava/lang/CharSequence;)V

    const-string v9, "id"

    invoke-virtual {v3, v9}, Lorg/json/JSONObject;->optInt(Ljava/lang/String;)I

    move-result v3

    invoke-static {v3}, Ljava/lang/Integer;->valueOf(I)Ljava/lang/Integer;

    move-result-object v3

    invoke-interface {v4, v3, v8}, Ljava/util/Map;->put(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;

    invoke-virtual {v0, v8}, Landroid/widget/LinearLayout;->addView(Landroid/view/View;)V

    .line 101
    :cond_1
    :goto_1
    add-int/lit8 v2, v2, 0x1

    goto :goto_0

    .line 105
    :cond_2
    new-instance v1, Landroid/widget/ScrollView;

    iget-object v2, p0, Lorg/garlic/client/LegacyUsers;->ui:Landroid/app/Activity;

    invoke-direct {v1, v2}, Landroid/widget/ScrollView;-><init>(Landroid/content/Context;)V

    invoke-virtual {v1, v0}, Landroid/widget/ScrollView;->addView(Landroid/view/View;)V

    .line 106
    new-instance v0, Landroid/app/AlertDialog$Builder;

    iget-object v2, p0, Lorg/garlic/client/LegacyUsers;->ui:Landroid/app/Activity;

    invoke-direct {v0, v2}, Landroid/app/AlertDialog$Builder;-><init>(Landroid/content/Context;)V

    const-string v2, "\u041d\u043e\u0432\u044b\u0439 \u043f\u043e\u043b\u044c\u0437\u043e\u0432\u0430\u0442\u0435\u043b\u044c Legacy"

    invoke-virtual {v0, v2}, Landroid/app/AlertDialog$Builder;->setTitle(Ljava/lang/CharSequence;)Landroid/app/AlertDialog$Builder;

    move-result-object v0

    invoke-virtual {v0, v1}, Landroid/app/AlertDialog$Builder;->setView(Landroid/view/View;)Landroid/app/AlertDialog$Builder;

    move-result-object v0

    const-string v1, "\u041e\u0442\u043c\u0435\u043d\u0430"

    const/4 v2, 0x0

    invoke-virtual {v0, v1, v2}, Landroid/app/AlertDialog$Builder;->setNegativeButton(Ljava/lang/CharSequence;Landroid/content/DialogInterface$OnClickListener;)Landroid/app/AlertDialog$Builder;

    move-result-object v0

    new-instance v2, Lorg/garlic/client/LegacyUsers$10;

    move-object v3, p0

    invoke-direct/range {v2 .. v7}, Lorg/garlic/client/LegacyUsers$10;-><init>(Lorg/garlic/client/LegacyUsers;Ljava/util/Map;Landroid/widget/EditText;Ljava/util/Calendar;Landroid/widget/CheckBox;)V

    .line 107
    const-string v1, "\u0421\u043e\u0437\u0434\u0430\u0442\u044c"

    invoke-virtual {v0, v1, v2}, Landroid/app/AlertDialog$Builder;->setPositiveButton(Ljava/lang/CharSequence;Landroid/content/DialogInterface$OnClickListener;)Landroid/app/AlertDialog$Builder;

    move-result-object v0

    .line 113
    invoke-virtual {v0}, Landroid/app/AlertDialog$Builder;->show()Landroid/app/AlertDialog;

    .line 114
    return-void
.end method

.method private error(Ljava/lang/String;)V
    .locals 2

    .line 24
    new-instance v0, Landroid/app/AlertDialog$Builder;

    iget-object v1, p0, Lorg/garlic/client/LegacyUsers;->ui:Landroid/app/Activity;

    invoke-direct {v0, v1}, Landroid/app/AlertDialog$Builder;-><init>(Landroid/content/Context;)V

    const-string v1, "\u041f\u043e\u043b\u044c\u0437\u043e\u0432\u0430\u0442\u0435\u043b\u0438 Legacy"

    invoke-virtual {v0, v1}, Landroid/app/AlertDialog$Builder;->setTitle(Ljava/lang/CharSequence;)Landroid/app/AlertDialog$Builder;

    move-result-object v0

    invoke-virtual {v0, p1}, Landroid/app/AlertDialog$Builder;->setMessage(Ljava/lang/CharSequence;)Landroid/app/AlertDialog$Builder;

    move-result-object p1

    .line 25
    const-string v0, "\u0417\u0430\u043a\u0440\u044b\u0442\u044c"

    const/4 v1, 0x0

    invoke-virtual {p1, v0, v1}, Landroid/app/AlertDialog$Builder;->setPositiveButton(Ljava/lang/CharSequence;Landroid/content/DialogInterface$OnClickListener;)Landroid/app/AlertDialog$Builder;

    move-result-object p1

    invoke-virtual {p1}, Landroid/app/AlertDialog$Builder;->show()Landroid/app/AlertDialog;

    .line 26
    return-void
.end method

.method static synthetic lambda$create$8(Landroid/widget/Button;Ljava/util/Calendar;)V
    .locals 2

    .line 96
    invoke-static {}, Ljava/text/DateFormat;->getDateInstance()Ljava/text/DateFormat;

    move-result-object v0

    invoke-virtual {p1}, Ljava/util/Calendar;->getTime()Ljava/util/Date;

    move-result-object p1

    invoke-virtual {v0, p1}, Ljava/text/DateFormat;->format(Ljava/util/Date;)Ljava/lang/String;

    move-result-object p1

    new-instance v0, Ljava/lang/StringBuilder;

    invoke-direct {v0}, Ljava/lang/StringBuilder;-><init>()V

    const-string v1, "\u0414\u043e "

    invoke-virtual {v0, v1}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v0

    invoke-virtual {v0, p1}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object p1

    invoke-virtual {p1}, Ljava/lang/StringBuilder;->toString()Ljava/lang/String;

    move-result-object p1

    invoke-virtual {p0, p1}, Landroid/widget/Button;->setText(Ljava/lang/CharSequence;)V

    return-void
.end method

.method static synthetic lambda$create$9(Ljava/util/Calendar;Ljava/lang/Runnable;Landroid/widget/DatePicker;III)V
    .locals 0

    .line 97
    invoke-virtual {p0, p3, p4, p5}, Ljava/util/Calendar;->set(III)V

    invoke-interface {p1}, Ljava/lang/Runnable;->run()V

    return-void
.end method

.method private options(Lorg/json/JSONObject;)V
    .locals 3

    .line 84
    const-string v0, "email"

    invoke-virtual {p1, v0}, Lorg/json/JSONObject;->optString(Ljava/lang/String;)Ljava/lang/String;

    move-result-object p1

    .line 85
    new-instance v0, Landroid/app/AlertDialog$Builder;

    iget-object v1, p0, Lorg/garlic/client/LegacyUsers;->ui:Landroid/app/Activity;

    invoke-direct {v0, v1}, Landroid/app/AlertDialog$Builder;-><init>(Landroid/content/Context;)V

    invoke-virtual {v0, p1}, Landroid/app/AlertDialog$Builder;->setTitle(Ljava/lang/CharSequence;)Landroid/app/AlertDialog$Builder;

    move-result-object v0

    const-string v1, "\u0421\u0441\u044b\u043b\u043a\u0430 \u043f\u043e\u0434\u043f\u0438\u0441\u043a\u0438"

    const-string v2, "\u041f\u0440\u043e\u0434\u043b\u0438\u0442\u044c \u043d\u0430 \u043d\u0435\u0441\u043a\u043e\u043b\u044c\u043a\u043e \u0434\u043d\u0435\u0439"

    filled-new-array {v1, v2}, [Ljava/lang/String;

    move-result-object v1

    new-instance v2, Lorg/garlic/client/LegacyUsers$7;

    invoke-direct {v2, p0, p1}, Lorg/garlic/client/LegacyUsers$7;-><init>(Lorg/garlic/client/LegacyUsers;Ljava/lang/String;)V

    invoke-virtual {v0, v1, v2}, Landroid/app/AlertDialog$Builder;->setItems([Ljava/lang/CharSequence;Landroid/content/DialogInterface$OnClickListener;)Landroid/app/AlertDialog$Builder;

    move-result-object p1

    .line 90
    invoke-virtual {p1}, Landroid/app/AlertDialog$Builder;->show()Landroid/app/AlertDialog;

    .line 91
    return-void
.end method

.method private request(Lorg/json/JSONObject;)V
    .locals 3

    .line 28
    iget-object v0, p0, Lorg/garlic/client/LegacyUsers;->active:Ljava/util/function/BooleanSupplier;

    invoke-interface {v0}, Ljava/util/function/BooleanSupplier;->getAsBoolean()Z

    move-result v0

    if-nez v0, :cond_0

    return-void

    .line 29
    :cond_0
    iget-object v0, p0, Lorg/garlic/client/LegacyUsers;->origin:Ljava/lang/String;

    .line 30
    iget-object v1, p0, Lorg/garlic/client/LegacyUsers;->executor:Ljava/util/concurrent/Executor;

    new-instance v2, Lorg/garlic/client/LegacyUsers$3;

    invoke-direct {v2, p0, p1, v0}, Lorg/garlic/client/LegacyUsers$3;-><init>(Lorg/garlic/client/LegacyUsers;Lorg/json/JSONObject;Ljava/lang/String;)V

    invoke-interface {v1, v2}, Ljava/util/concurrent/Executor;->execute(Ljava/lang/Runnable;)V

    .line 57
    return-void
.end method

.method private route(Lorg/json/JSONObject;)Ljava/lang/String;
    .locals 2

    .line 58
    const-string v0, "protocol"

    invoke-virtual {p1, v0}, Lorg/json/JSONObject;->optString(Ljava/lang/String;)Ljava/lang/String;

    move-result-object v0

    const-string v1, "port"

    invoke-virtual {p1, v1}, Lorg/json/JSONObject;->optInt(Ljava/lang/String;)I

    move-result p1

    new-instance v1, Ljava/lang/StringBuilder;

    invoke-direct {v1}, Ljava/lang/StringBuilder;-><init>()V

    invoke-virtual {v1, v0}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v0

    const-string v1, " / "

    invoke-virtual {v0, v1}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v0

    invoke-virtual {v0, p1}, Ljava/lang/StringBuilder;->append(I)Ljava/lang/StringBuilder;

    move-result-object p1

    invoke-virtual {p1}, Ljava/lang/StringBuilder;->toString()Ljava/lang/String;

    move-result-object p1

    return-object p1
.end method

.method private send(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;Ljava/lang/Object;)V
    .locals 2

    .line 79
    :try_start_0
    new-instance v0, Lorg/json/JSONObject;

    invoke-direct {v0}, Lorg/json/JSONObject;-><init>()V

    const-string v1, "action"

    invoke-virtual {v0, v1, p1}, Lorg/json/JSONObject;->put(Ljava/lang/String;Ljava/lang/Object;)Lorg/json/JSONObject;

    move-result-object p1

    const-string v0, "email"

    invoke-virtual {p1, v0, p2}, Lorg/json/JSONObject;->put(Ljava/lang/String;Ljava/lang/Object;)Lorg/json/JSONObject;

    move-result-object p1

    .line 80
    if-eqz p3, :cond_0

    invoke-virtual {p1, p3, p4}, Lorg/json/JSONObject;->put(Ljava/lang/String;Ljava/lang/Object;)Lorg/json/JSONObject;

    :cond_0
    invoke-direct {p0, p1}, Lorg/garlic/client/LegacyUsers;->request(Lorg/json/JSONObject;)V
    :try_end_0
    .catch Lorg/json/JSONException; {:try_start_0 .. :try_end_0} :catch_0

    .line 81
    goto :goto_0

    :catch_0
    move-exception p1

    const-string p1, "\u041f\u0440\u043e\u0432\u0435\u0440\u044c\u0442\u0435 \u0432\u0432\u0435\u0434\u0451\u043d\u043d\u044b\u0435 \u0434\u0430\u043d\u043d\u044b\u0435"

    invoke-direct {p0, p1}, Lorg/garlic/client/LegacyUsers;->error(Ljava/lang/String;)V

    .line 82
    :goto_0
    return-void
.end method

.method private showList()V
    .locals 16
    .annotation system Ldalvik/annotation/Throws;
        value = {
            Lorg/json/JSONException;
        }
    .end annotation

    .line 60
    move-object/from16 v0, p0

    iget-object v1, v0, Lorg/garlic/client/LegacyUsers;->directory:Lorg/json/JSONObject;

    const-string v2, "users"

    invoke-virtual {v1, v2}, Lorg/json/JSONObject;->getJSONArray(Ljava/lang/String;)Lorg/json/JSONArray;

    move-result-object v1

    iget-object v2, v0, Lorg/garlic/client/LegacyUsers;->directory:Lorg/json/JSONObject;

    const-string v3, "routes"

    invoke-virtual {v2, v3}, Lorg/json/JSONObject;->getJSONArray(Ljava/lang/String;)Lorg/json/JSONArray;

    move-result-object v2

    .line 61
    new-instance v3, Ljava/util/HashMap;

    invoke-direct {v3}, Ljava/util/HashMap;-><init>()V

    .line 62
    const/4 v5, 0x0

    :goto_0
    invoke-virtual {v2}, Lorg/json/JSONArray;->length()I

    move-result v6

    if-ge v5, v6, :cond_0

    invoke-virtual {v2, v5}, Lorg/json/JSONArray;->getJSONObject(I)Lorg/json/JSONObject;

    move-result-object v6

    const-string v7, "id"

    invoke-virtual {v6, v7}, Lorg/json/JSONObject;->getInt(Ljava/lang/String;)I

    move-result v7

    invoke-static {v7}, Ljava/lang/Integer;->valueOf(I)Ljava/lang/Integer;

    move-result-object v7

    invoke-direct {v0, v6}, Lorg/garlic/client/LegacyUsers;->route(Lorg/json/JSONObject;)Ljava/lang/String;

    move-result-object v6

    invoke-interface {v3, v7, v6}, Ljava/util/Map;->put(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;

    add-int/lit8 v5, v5, 0x1

    goto :goto_0

    .line 63
    :cond_0
    invoke-virtual {v1}, Lorg/json/JSONArray;->length()I

    move-result v2

    new-array v5, v2, [Ljava/lang/String;

    .line 64
    const/4 v6, 0x0

    :goto_1
    if-ge v6, v2, :cond_6

    .line 65
    invoke-virtual {v1, v6}, Lorg/json/JSONArray;->getJSONObject(I)Lorg/json/JSONObject;

    move-result-object v7

    const-string v8, "traffic"

    invoke-virtual {v7, v8}, Lorg/json/JSONObject;->optJSONObject(Ljava/lang/String;)Lorg/json/JSONObject;

    move-result-object v8

    .line 66
    const-string v9, "expiryTime"

    invoke-virtual {v7, v9}, Lorg/json/JSONObject;->getLong(Ljava/lang/String;)J

    move-result-wide v9

    .line 67
    const-wide/16 v11, 0x0

    cmp-long v11, v9, v11

    if-nez v11, :cond_1

    const-string v9, "\u0411\u0435\u0437 \u0441\u0440\u043e\u043a\u0430"

    goto :goto_2

    :cond_1
    if-gez v11, :cond_2

    const-string v9, "\u0421\u0440\u043e\u043a \u0441 \u043f\u0435\u0440\u0432\u043e\u0433\u043e \u043f\u043e\u0434\u043a\u043b\u044e\u0447\u0435\u043d\u0438\u044f"

    goto :goto_2

    :cond_2
    invoke-static {}, Ljava/text/DateFormat;->getDateInstance()Ljava/text/DateFormat;

    move-result-object v11

    new-instance v12, Ljava/util/Date;

    invoke-direct {v12, v9, v10}, Ljava/util/Date;-><init>(J)V

    invoke-virtual {v11, v12}, Ljava/text/DateFormat;->format(Ljava/util/Date;)Ljava/lang/String;

    move-result-object v9

    new-instance v10, Ljava/lang/StringBuilder;

    invoke-direct {v10}, Ljava/lang/StringBuilder;-><init>()V

    const-string v11, "\u0414\u043e "

    invoke-virtual {v10, v11}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v10

    invoke-virtual {v10, v9}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v9

    invoke-virtual {v9}, Ljava/lang/StringBuilder;->toString()Ljava/lang/String;

    move-result-object v9

    .line 68
    :goto_2
    new-instance v10, Ljava/util/ArrayList;

    invoke-direct {v10}, Ljava/util/ArrayList;-><init>()V

    .line 69
    const-string v11, "inboundIds"

    invoke-virtual {v7, v11}, Lorg/json/JSONObject;->getJSONArray(Ljava/lang/String;)Lorg/json/JSONArray;

    move-result-object v11

    .line 70
    const/4 v12, 0x0

    :goto_3
    invoke-virtual {v11}, Lorg/json/JSONArray;->length()I

    move-result v13

    if-ge v12, v13, :cond_3

    invoke-virtual {v11, v12}, Lorg/json/JSONArray;->getInt(I)I

    move-result v13

    invoke-static {v13}, Ljava/lang/Integer;->valueOf(I)Ljava/lang/Integer;

    move-result-object v13

    invoke-virtual {v11, v12}, Lorg/json/JSONArray;->getInt(I)I

    move-result v14

    new-instance v15, Ljava/lang/StringBuilder;

    invoke-direct {v15}, Ljava/lang/StringBuilder;-><init>()V

    const-string v4, "#"

    invoke-virtual {v15, v4}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v4

    invoke-virtual {v4, v14}, Ljava/lang/StringBuilder;->append(I)Ljava/lang/StringBuilder;

    move-result-object v4

    invoke-virtual {v4}, Ljava/lang/StringBuilder;->toString()Ljava/lang/String;

    move-result-object v4

    invoke-interface {v3, v13, v4}, Ljava/util/Map;->getOrDefault(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;

    move-result-object v4

    check-cast v4, Ljava/lang/String;

    invoke-interface {v10, v4}, Ljava/util/List;->add(Ljava/lang/Object;)Z

    add-int/lit8 v12, v12, 0x1

    goto :goto_3

    .line 71
    :cond_3
    const-string v4, "email"

    invoke-virtual {v7, v4}, Lorg/json/JSONObject;->getString(Ljava/lang/String;)Ljava/lang/String;

    move-result-object v4

    const-string v11, "enable"

    invoke-virtual {v7, v11}, Lorg/json/JSONObject;->getBoolean(Ljava/lang/String;)Z

    move-result v7

    const-string v11, ""

    if-eqz v7, :cond_4

    move-object v7, v11

    goto :goto_4

    :cond_4
    const-string v7, " \u00b7 \u043e\u0442\u043a\u043b\u044e\u0447\u0451\u043d"

    .line 72
    :goto_4
    if-nez v8, :cond_5

    goto :goto_5

    :cond_5
    iget-object v11, v0, Lorg/garlic/client/LegacyUsers;->ui:Landroid/app/Activity;

    const-string v12, "up"

    invoke-virtual {v8, v12}, Lorg/json/JSONObject;->optLong(Ljava/lang/String;)J

    move-result-wide v12

    const-string v14, "down"

    invoke-virtual {v8, v14}, Lorg/json/JSONObject;->optLong(Ljava/lang/String;)J

    move-result-wide v14

    add-long/2addr v12, v14

    invoke-static {v11, v12, v13}, Landroid/text/format/Formatter;->formatFileSize(Landroid/content/Context;J)Ljava/lang/String;

    move-result-object v8

    new-instance v11, Ljava/lang/StringBuilder;

    invoke-direct {v11}, Ljava/lang/StringBuilder;-><init>()V

    const-string v12, " \u00b7 "

    invoke-virtual {v11, v12}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v11

    invoke-virtual {v11, v8}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v8

    invoke-virtual {v8}, Ljava/lang/StringBuilder;->toString()Ljava/lang/String;

    move-result-object v11

    .line 73
    :goto_5
    const-string v8, ", "

    invoke-static {v8, v10}, Ljava/lang/String;->join(Ljava/lang/CharSequence;Ljava/lang/Iterable;)Ljava/lang/String;

    move-result-object v8

    new-instance v10, Ljava/lang/StringBuilder;

    invoke-direct {v10}, Ljava/lang/StringBuilder;-><init>()V

    invoke-virtual {v10, v4}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v4

    invoke-virtual {v4, v7}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v4

    const-string v7, "\n"

    invoke-virtual {v4, v7}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v4

    invoke-virtual {v4, v9}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v4

    invoke-virtual {v4, v11}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v4

    invoke-virtual {v4, v7}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v4

    invoke-virtual {v4, v8}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v4

    invoke-virtual {v4}, Ljava/lang/StringBuilder;->toString()Ljava/lang/String;

    move-result-object v4

    aput-object v4, v5, v6

    .line 64
    add-int/lit8 v6, v6, 0x1

    goto/16 :goto_1

    .line 75
    :cond_6
    new-instance v2, Landroid/app/AlertDialog$Builder;

    iget-object v3, v0, Lorg/garlic/client/LegacyUsers;->ui:Landroid/app/Activity;

    invoke-direct {v2, v3}, Landroid/app/AlertDialog$Builder;-><init>(Landroid/content/Context;)V

    const-string v3, "\u041f\u043e\u043b\u044c\u0437\u043e\u0432\u0430\u0442\u0435\u043b\u0438 Legacy"

    invoke-virtual {v2, v3}, Landroid/app/AlertDialog$Builder;->setTitle(Ljava/lang/CharSequence;)Landroid/app/AlertDialog$Builder;

    move-result-object v2

    new-instance v3, Lorg/garlic/client/LegacyUsers$4;

    invoke-direct {v3, v0, v1}, Lorg/garlic/client/LegacyUsers$4;-><init>(Lorg/garlic/client/LegacyUsers;Lorg/json/JSONArray;)V

    invoke-virtual {v2, v5, v3}, Landroid/app/AlertDialog$Builder;->setItems([Ljava/lang/CharSequence;Landroid/content/DialogInterface$OnClickListener;)Landroid/app/AlertDialog$Builder;

    move-result-object v1

    new-instance v2, Lorg/garlic/client/LegacyUsers$5;

    invoke-direct {v2, v0}, Lorg/garlic/client/LegacyUsers$5;-><init>(Lorg/garlic/client/LegacyUsers;)V

    .line 76
    const-string v0, "\u0421\u043e\u0437\u0434\u0430\u0442\u044c"

    invoke-virtual {v1, v0, v2}, Landroid/app/AlertDialog$Builder;->setPositiveButton(Ljava/lang/CharSequence;Landroid/content/DialogInterface$OnClickListener;)Landroid/app/AlertDialog$Builder;

    move-result-object v0

    const-string v1, "\u0417\u0430\u043a\u0440\u044b\u0442\u044c"

    const/4 v2, 0x0

    invoke-virtual {v0, v1, v2}, Landroid/app/AlertDialog$Builder;->setNegativeButton(Ljava/lang/CharSequence;Landroid/content/DialogInterface$OnClickListener;)Landroid/app/AlertDialog$Builder;

    move-result-object v0

    invoke-virtual {v0}, Landroid/app/AlertDialog$Builder;->show()Landroid/app/AlertDialog;

    .line 77
    return-void
.end method


# virtual methods
.method synthetic lambda$create$10$org-garlic-client-LegacyUsers(Ljava/util/Calendar;Ljava/lang/Runnable;Landroid/view/View;)V
    .locals 6

    .line 97
    new-instance v0, Landroid/app/DatePickerDialog;

    iget-object v1, p0, Lorg/garlic/client/LegacyUsers;->ui:Landroid/app/Activity;

    new-instance v2, Lorg/garlic/client/LegacyUsers$0;

    invoke-direct {v2, p1, p2}, Lorg/garlic/client/LegacyUsers$0;-><init>(Ljava/util/Calendar;Ljava/lang/Runnable;)V

    .line 98
    const/4 p2, 0x1

    invoke-virtual {p1, p2}, Ljava/util/Calendar;->get(I)I

    move-result v3

    const/4 p2, 0x2

    invoke-virtual {p1, p2}, Ljava/util/Calendar;->get(I)I

    move-result v4

    const/4 p2, 0x5

    invoke-virtual {p1, p2}, Ljava/util/Calendar;->get(I)I

    move-result v5

    invoke-direct/range {v0 .. v5}, Landroid/app/DatePickerDialog;-><init>(Landroid/content/Context;Landroid/app/DatePickerDialog$OnDateSetListener;III)V

    invoke-virtual {v0}, Landroid/app/DatePickerDialog;->show()V

    .line 97
    return-void
.end method

.method synthetic lambda$create$11$org-garlic-client-LegacyUsers(Ljava/util/Map;Landroid/widget/EditText;Ljava/util/Calendar;Landroid/widget/CheckBox;Landroid/content/DialogInterface;I)V
    .locals 1

    .line 108
    :try_start_0
    new-instance p5, Lorg/json/JSONArray;

    invoke-direct {p5}, Lorg/json/JSONArray;-><init>()V

    invoke-interface {p1}, Ljava/util/Map;->entrySet()Ljava/util/Set;

    move-result-object p1

    invoke-interface {p1}, Ljava/util/Set;->iterator()Ljava/util/Iterator;

    move-result-object p1

    :cond_0
    :goto_0
    invoke-interface {p1}, Ljava/util/Iterator;->hasNext()Z

    move-result p6

    if-eqz p6, :cond_1

    invoke-interface {p1}, Ljava/util/Iterator;->next()Ljava/lang/Object;

    move-result-object p6

    check-cast p6, Ljava/util/Map$Entry;

    invoke-interface {p6}, Ljava/util/Map$Entry;->getValue()Ljava/lang/Object;

    move-result-object v0

    check-cast v0, Landroid/widget/CheckBox;

    invoke-virtual {v0}, Landroid/widget/CheckBox;->isChecked()Z

    move-result v0

    if-eqz v0, :cond_0

    invoke-interface {p6}, Ljava/util/Map$Entry;->getKey()Ljava/lang/Object;

    move-result-object p6

    check-cast p6, Ljava/lang/Integer;

    invoke-virtual {p6}, Ljava/lang/Integer;->intValue()I

    move-result p6

    invoke-virtual {p5, p6}, Lorg/json/JSONArray;->put(I)Lorg/json/JSONArray;

    goto :goto_0

    .line 109
    :cond_1
    invoke-virtual {p5}, Lorg/json/JSONArray;->length()I

    move-result p1

    if-eqz p1, :cond_4

    invoke-virtual {p2}, Landroid/widget/EditText;->getText()Landroid/text/Editable;

    move-result-object p1

    invoke-virtual {p1}, Ljava/lang/Object;->toString()Ljava/lang/String;

    move-result-object p1

    invoke-virtual {p1}, Ljava/lang/String;->trim()Ljava/lang/String;

    move-result-object p1

    invoke-virtual {p1}, Ljava/lang/String;->isEmpty()Z

    move-result p1

    if-eqz p1, :cond_2

    goto :goto_2

    .line 110
    :cond_2
    const/16 p1, 0xb

    const/16 p6, 0x17

    invoke-virtual {p3, p1, p6}, Ljava/util/Calendar;->set(II)V

    const/16 p1, 0xc

    const/16 p6, 0x3b

    invoke-virtual {p3, p1, p6}, Ljava/util/Calendar;->set(II)V

    const/16 p1, 0xd

    invoke-virtual {p3, p1, p6}, Ljava/util/Calendar;->set(II)V

    const/16 p1, 0xe

    const/16 p6, 0x3e7

    invoke-virtual {p3, p1, p6}, Ljava/util/Calendar;->set(II)V

    .line 111
    new-instance p1, Lorg/json/JSONObject;

    invoke-direct {p1}, Lorg/json/JSONObject;-><init>()V

    const-string p6, "action"

    const-string v0, "create"

    invoke-virtual {p1, p6, v0}, Lorg/json/JSONObject;->put(Ljava/lang/String;Ljava/lang/Object;)Lorg/json/JSONObject;

    move-result-object p1

    const-string p6, "email"

    invoke-virtual {p2}, Landroid/widget/EditText;->getText()Landroid/text/Editable;

    move-result-object p2

    invoke-virtual {p2}, Ljava/lang/Object;->toString()Ljava/lang/String;

    move-result-object p2

    invoke-virtual {p2}, Ljava/lang/String;->trim()Ljava/lang/String;

    move-result-object p2

    invoke-virtual {p1, p6, p2}, Lorg/json/JSONObject;->put(Ljava/lang/String;Ljava/lang/Object;)Lorg/json/JSONObject;

    move-result-object p1

    const-string p2, "expiry_ms"

    .line 112
    invoke-virtual {p4}, Landroid/widget/CheckBox;->isChecked()Z

    move-result p4

    if-eqz p4, :cond_3

    const-wide/16 p3, 0x0

    goto :goto_1

    :cond_3
    invoke-virtual {p3}, Ljava/util/Calendar;->getTimeInMillis()J

    move-result-wide p3

    :goto_1
    invoke-virtual {p1, p2, p3, p4}, Lorg/json/JSONObject;->put(Ljava/lang/String;J)Lorg/json/JSONObject;

    move-result-object p1

    const-string p2, "inbound_ids"

    invoke-virtual {p1, p2, p5}, Lorg/json/JSONObject;->put(Ljava/lang/String;Ljava/lang/Object;)Lorg/json/JSONObject;

    move-result-object p1

    .line 111
    invoke-direct {p0, p1}, Lorg/garlic/client/LegacyUsers;->request(Lorg/json/JSONObject;)V

    .line 113
    goto :goto_3

    .line 109
    :cond_4
    :goto_2
    const-string p1, "\u0412\u0432\u0435\u0434\u0438\u0442\u0435 \u0438\u043c\u044f \u0438 \u0432\u044b\u0431\u0435\u0440\u0438\u0442\u0435 \u0432\u0445\u043e\u0434 W"

    invoke-direct {p0, p1}, Lorg/garlic/client/LegacyUsers;->error(Ljava/lang/String;)V
    :try_end_0
    .catch Lorg/json/JSONException; {:try_start_0 .. :try_end_0} :catch_0

    return-void

    .line 113
    :catch_0
    move-exception p1

    const-string p1, "\u041d\u0435 \u0443\u0434\u0430\u043b\u043e\u0441\u044c \u0441\u043e\u0437\u0434\u0430\u0442\u044c \u0437\u0430\u043f\u0440\u043e\u0441"

    invoke-direct {p0, p1}, Lorg/garlic/client/LegacyUsers;->error(Ljava/lang/String;)V

    :goto_3
    return-void
.end method

.method synthetic lambda$options$6$org-garlic-client-LegacyUsers(Ljava/lang/String;Landroid/widget/EditText;Landroid/content/DialogInterface;I)V
    .locals 0

    .line 89
    :try_start_0
    const-string p3, "extend"

    const-string p4, "days"

    invoke-virtual {p2}, Landroid/widget/EditText;->getText()Landroid/text/Editable;

    move-result-object p2

    invoke-virtual {p2}, Ljava/lang/Object;->toString()Ljava/lang/String;

    move-result-object p2

    invoke-static {p2}, Ljava/lang/Integer;->parseInt(Ljava/lang/String;)I

    move-result p2

    invoke-static {p2}, Ljava/lang/Integer;->valueOf(I)Ljava/lang/Integer;

    move-result-object p2

    invoke-direct {p0, p3, p1, p4, p2}, Lorg/garlic/client/LegacyUsers;->send(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;Ljava/lang/Object;)V
    :try_end_0
    .catch Ljava/lang/NumberFormatException; {:try_start_0 .. :try_end_0} :catch_0

    goto :goto_0

    :catch_0
    move-exception p1

    const-string p1, "\u0412\u0432\u0435\u0434\u0438\u0442\u0435 \u0447\u0438\u0441\u043b\u043e \u0434\u043d\u0435\u0439"

    invoke-direct {p0, p1}, Lorg/garlic/client/LegacyUsers;->error(Ljava/lang/String;)V

    :goto_0
    return-void
.end method

.method synthetic lambda$options$7$org-garlic-client-LegacyUsers(Ljava/lang/String;Landroid/content/DialogInterface;I)V
    .locals 2

    .line 86
    const/4 p2, 0x0

    if-nez p3, :cond_0

    const-string v0, "subscription"

    invoke-direct {p0, v0, p1, p2, p2}, Lorg/garlic/client/LegacyUsers;->send(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;Ljava/lang/Object;)V

    .line 87
    :cond_0
    const/4 v0, 0x1

    if-ne p3, v0, :cond_1

    new-instance p3, Landroid/widget/EditText;

    iget-object v0, p0, Lorg/garlic/client/LegacyUsers;->ui:Landroid/app/Activity;

    invoke-direct {p3, v0}, Landroid/widget/EditText;-><init>(Landroid/content/Context;)V

    const/4 v0, 0x2

    invoke-virtual {p3, v0}, Landroid/widget/EditText;->setInputType(I)V

    const-string v0, "\u0414\u043d\u0435\u0439"

    invoke-virtual {p3, v0}, Landroid/widget/EditText;->setHint(Ljava/lang/CharSequence;)V

    .line 88
    new-instance v0, Landroid/app/AlertDialog$Builder;

    iget-object v1, p0, Lorg/garlic/client/LegacyUsers;->ui:Landroid/app/Activity;

    invoke-direct {v0, v1}, Landroid/app/AlertDialog$Builder;-><init>(Landroid/content/Context;)V

    const-string v1, "\u041f\u0440\u043e\u0434\u043b\u0435\u043d\u0438\u0435 \u043f\u043e\u0434\u043f\u0438\u0441\u043a\u0438"

    invoke-virtual {v0, v1}, Landroid/app/AlertDialog$Builder;->setTitle(Ljava/lang/CharSequence;)Landroid/app/AlertDialog$Builder;

    move-result-object v0

    invoke-virtual {v0, p3}, Landroid/app/AlertDialog$Builder;->setView(Landroid/view/View;)Landroid/app/AlertDialog$Builder;

    move-result-object v0

    const-string v1, "\u041e\u0442\u043c\u0435\u043d\u0430"

    invoke-virtual {v0, v1, p2}, Landroid/app/AlertDialog$Builder;->setNegativeButton(Ljava/lang/CharSequence;Landroid/content/DialogInterface$OnClickListener;)Landroid/app/AlertDialog$Builder;

    move-result-object p2

    new-instance v0, Lorg/garlic/client/LegacyUsers$11;

    invoke-direct {v0, p0, p1, p3}, Lorg/garlic/client/LegacyUsers$11;-><init>(Lorg/garlic/client/LegacyUsers;Ljava/lang/String;Landroid/widget/EditText;)V

    .line 89
    const-string p1, "\u041f\u0440\u043e\u0434\u043b\u0438\u0442\u044c"

    invoke-virtual {p2, p1, v0}, Landroid/app/AlertDialog$Builder;->setPositiveButton(Ljava/lang/CharSequence;Landroid/content/DialogInterface$OnClickListener;)Landroid/app/AlertDialog$Builder;

    move-result-object p1

    invoke-virtual {p1}, Landroid/app/AlertDialog$Builder;->show()Landroid/app/AlertDialog;

    .line 90
    :cond_1
    return-void
.end method

.method synthetic lambda$request$0$org-garlic-client-LegacyUsers(Ljava/lang/String;Landroid/content/DialogInterface;I)V
    .locals 0

    .line 46
    iget-object p2, p0, Lorg/garlic/client/LegacyUsers;->ui:Landroid/app/Activity;

    const-string p3, "clipboard"

    invoke-virtual {p2, p3}, Landroid/app/Activity;->getSystemService(Ljava/lang/String;)Ljava/lang/Object;

    move-result-object p2

    check-cast p2, Landroid/content/ClipboardManager;

    .line 47
    const-string p3, "\u041f\u043e\u0434\u043f\u0438\u0441\u043a\u0430 Legacy"

    invoke-static {p3, p1}, Landroid/content/ClipData;->newPlainText(Ljava/lang/CharSequence;Ljava/lang/CharSequence;)Landroid/content/ClipData;

    move-result-object p1

    invoke-virtual {p2, p1}, Landroid/content/ClipboardManager;->setPrimaryClip(Landroid/content/ClipData;)V

    .line 48
    return-void
.end method

.method synthetic lambda$request$1$org-garlic-client-LegacyUsers(Lorg/json/JSONObject;Lorg/json/JSONObject;)V
    .locals 2

    .line 37
    const-string v0, "url"

    iget-object v1, p0, Lorg/garlic/client/LegacyUsers;->active:Ljava/util/function/BooleanSupplier;

    invoke-interface {v1}, Ljava/util/function/BooleanSupplier;->getAsBoolean()Z

    move-result v1

    if-nez v1, :cond_0

    return-void

    .line 39
    :cond_0
    :try_start_0
    const-string v1, "ok"

    invoke-virtual {p1, v1}, Lorg/json/JSONObject;->optBoolean(Ljava/lang/String;)Z

    move-result v1

    if-nez v1, :cond_1

    const-string p2, "error"

    const-string v0, "\u0421\u0435\u0440\u0432\u0435\u0440 \u043d\u0435\u0434\u043e\u0441\u0442\u0443\u043f\u0435\u043d"

    invoke-virtual {p1, p2, v0}, Lorg/json/JSONObject;->optString(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;

    move-result-object p1

    invoke-direct {p0, p1}, Lorg/garlic/client/LegacyUsers;->error(Ljava/lang/String;)V

    return-void

    .line 40
    :cond_1
    if-nez p2, :cond_2

    .line 41
    const-string p2, "origin"

    invoke-virtual {p1, p2}, Lorg/json/JSONObject;->getString(Ljava/lang/String;)Ljava/lang/String;

    move-result-object p2

    iput-object p2, p0, Lorg/garlic/client/LegacyUsers;->origin:Ljava/lang/String;

    const-string p2, "legacy"

    invoke-virtual {p1, p2}, Lorg/json/JSONObject;->getJSONObject(Ljava/lang/String;)Lorg/json/JSONObject;

    move-result-object p1

    iput-object p1, p0, Lorg/garlic/client/LegacyUsers;->directory:Lorg/json/JSONObject;

    invoke-direct {p0}, Lorg/garlic/client/LegacyUsers;->showList()V

    goto :goto_0

    .line 42
    :cond_2
    invoke-virtual {p1, v0}, Lorg/json/JSONObject;->has(Ljava/lang/String;)Z

    move-result p2

    if-eqz p2, :cond_3

    .line 43
    invoke-virtual {p1, v0}, Lorg/json/JSONObject;->getString(Ljava/lang/String;)Ljava/lang/String;

    move-result-object p1

    .line 44
    new-instance p2, Landroid/app/AlertDialog$Builder;

    iget-object v0, p0, Lorg/garlic/client/LegacyUsers;->ui:Landroid/app/Activity;

    invoke-direct {p2, v0}, Landroid/app/AlertDialog$Builder;-><init>(Landroid/content/Context;)V

    const-string v0, "\u041f\u043e\u0434\u043f\u0438\u0441\u043a\u0430 T1"

    invoke-virtual {p2, v0}, Landroid/app/AlertDialog$Builder;->setTitle(Ljava/lang/CharSequence;)Landroid/app/AlertDialog$Builder;

    move-result-object p2

    invoke-virtual {p2, p1}, Landroid/app/AlertDialog$Builder;->setMessage(Ljava/lang/CharSequence;)Landroid/app/AlertDialog$Builder;

    move-result-object p2

    const-string v0, "\u0421\u043a\u043e\u043f\u0438\u0440\u043e\u0432\u0430\u0442\u044c"

    new-instance v1, Lorg/garlic/client/LegacyUsers$6;

    invoke-direct {v1, p0, p1}, Lorg/garlic/client/LegacyUsers$6;-><init>(Lorg/garlic/client/LegacyUsers;Ljava/lang/String;)V

    .line 45
    invoke-virtual {p2, v0, v1}, Landroid/app/AlertDialog$Builder;->setPositiveButton(Ljava/lang/CharSequence;Landroid/content/DialogInterface$OnClickListener;)Landroid/app/AlertDialog$Builder;

    move-result-object p1

    const-string p2, "\u0417\u0430\u043a\u0440\u044b\u0442\u044c"

    .line 48
    const/4 v0, 0x0

    invoke-virtual {p1, p2, v0}, Landroid/app/AlertDialog$Builder;->setNegativeButton(Ljava/lang/CharSequence;Landroid/content/DialogInterface$OnClickListener;)Landroid/app/AlertDialog$Builder;

    move-result-object p1

    invoke-virtual {p1}, Landroid/app/AlertDialog$Builder;->show()Landroid/app/AlertDialog;

    .line 49
    goto :goto_0

    .line 50
    :cond_3
    const-string p2, "nodePending"

    invoke-virtual {p1, p2}, Lorg/json/JSONObject;->optBoolean(Ljava/lang/String;)Z

    move-result p1

    if-eqz p1, :cond_4

    iget-object p1, p0, Lorg/garlic/client/LegacyUsers;->ui:Landroid/app/Activity;

    const-string p2, "T1 \u0441\u043e\u0445\u0440\u0430\u043d\u0438\u043b \u0438\u0437\u043c\u0435\u043d\u0435\u043d\u0438\u044f. W \u043e\u0436\u0438\u0434\u0430\u0435\u0442 \u0441\u0438\u043d\u0445\u0440\u043e\u043d\u0438\u0437\u0430\u0446\u0438\u0438"

    const/4 v0, 0x1

    invoke-static {p1, p2, v0}, Landroid/widget/Toast;->makeText(Landroid/content/Context;Ljava/lang/CharSequence;I)Landroid/widget/Toast;

    move-result-object p1

    invoke-virtual {p1}, Landroid/widget/Toast;->show()V

    .line 51
    :cond_4
    invoke-virtual {p0}, Lorg/garlic/client/LegacyUsers;->open()V
    :try_end_0
    .catch Lorg/json/JSONException; {:try_start_0 .. :try_end_0} :catch_0

    .line 53
    :goto_0
    goto :goto_1

    :catch_0
    move-exception p1

    const-string p1, "\u041d\u0435\u043a\u043e\u0440\u0440\u0435\u043a\u0442\u043d\u044b\u0439 \u043e\u0442\u0432\u0435\u0442 \u0441\u0435\u0440\u0432\u0435\u0440\u0430 Legacy"

    invoke-direct {p0, p1}, Lorg/garlic/client/LegacyUsers;->error(Ljava/lang/String;)V

    .line 54
    :goto_1
    return-void
.end method

.method synthetic lambda$request$2$org-garlic-client-LegacyUsers()V
    .locals 1

    .line 55
    iget-object v0, p0, Lorg/garlic/client/LegacyUsers;->active:Ljava/util/function/BooleanSupplier;

    invoke-interface {v0}, Ljava/util/function/BooleanSupplier;->getAsBoolean()Z

    move-result v0

    if-eqz v0, :cond_0

    const-string v0, "\u041d\u0435 \u0443\u0434\u0430\u043b\u043e\u0441\u044c \u043f\u043e\u043b\u0443\u0447\u0438\u0442\u044c \u043e\u0442\u0432\u0435\u0442. \u041e\u0431\u043d\u043e\u0432\u0438\u0442\u0435 \u0441\u043f\u0438\u0441\u043e\u043a \u043f\u0435\u0440\u0435\u0434 \u043f\u043e\u0432\u0442\u043e\u0440\u043e\u043c"

    invoke-direct {p0, v0}, Lorg/garlic/client/LegacyUsers;->error(Ljava/lang/String;)V

    :cond_0
    return-void
.end method

.method synthetic lambda$request$3$org-garlic-client-LegacyUsers(Lorg/json/JSONObject;Ljava/lang/String;)V
    .locals 3

    .line 32
    :try_start_0
    new-instance v0, Lorg/json/JSONObject;

    invoke-direct {v0}, Lorg/json/JSONObject;-><init>()V

    .line 33
    if-eqz p1, :cond_0

    const-string v1, "origin"

    invoke-virtual {v0, v1, p2}, Lorg/json/JSONObject;->put(Ljava/lang/String;Ljava/lang/Object;)Lorg/json/JSONObject;

    move-result-object p2

    const-string v1, "operation"

    invoke-virtual {p2, v1, p1}, Lorg/json/JSONObject;->put(Ljava/lang/String;Ljava/lang/Object;)Lorg/json/JSONObject;

    .line 34
    :cond_0
    new-instance p2, Lorg/json/JSONObject;

    invoke-direct {p2}, Lorg/json/JSONObject;-><init>()V

    const-string v1, "legacy"

    invoke-virtual {p2, v1, v0}, Lorg/json/JSONObject;->put(Ljava/lang/String;Ljava/lang/Object;)Lorg/json/JSONObject;

    move-result-object p2

    invoke-virtual {p2}, Lorg/json/JSONObject;->toString()Ljava/lang/String;

    move-result-object p2

    .line 35
    new-instance v0, Lorg/json/JSONObject;

    iget-object v1, p0, Lorg/garlic/client/LegacyUsers;->dir:Ljava/lang/String;

    iget-object v2, p0, Lorg/garlic/client/LegacyUsers;->origins:Ljava/lang/String;

    invoke-static {v1, v2, p2}, Lorg/garlic/client/GarlicBridge;->adminNetwork(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;

    move-result-object p2

    invoke-direct {v0, p2}, Lorg/json/JSONObject;-><init>(Ljava/lang/String;)V

    .line 36
    iget-object p2, p0, Lorg/garlic/client/LegacyUsers;->ui:Landroid/app/Activity;

    new-instance v1, Lorg/garlic/client/LegacyUsers$1;

    invoke-direct {v1, p0, v0, p1}, Lorg/garlic/client/LegacyUsers$1;-><init>(Lorg/garlic/client/LegacyUsers;Lorg/json/JSONObject;Lorg/json/JSONObject;)V

    invoke-virtual {p2, v1}, Landroid/app/Activity;->runOnUiThread(Ljava/lang/Runnable;)V
    :try_end_0
    .catch Ljava/lang/Exception; {:try_start_0 .. :try_end_0} :catch_0

    .line 55
    goto :goto_0

    :catch_0
    move-exception p1

    iget-object p1, p0, Lorg/garlic/client/LegacyUsers;->ui:Landroid/app/Activity;

    new-instance p2, Lorg/garlic/client/LegacyUsers$2;

    invoke-direct {p2, p0}, Lorg/garlic/client/LegacyUsers$2;-><init>(Lorg/garlic/client/LegacyUsers;)V

    invoke-virtual {p1, p2}, Landroid/app/Activity;->runOnUiThread(Ljava/lang/Runnable;)V

    .line 56
    :goto_0
    return-void
.end method

.method synthetic lambda$showList$4$org-garlic-client-LegacyUsers(Lorg/json/JSONArray;Landroid/content/DialogInterface;I)V
    .locals 0

    .line 75
    invoke-virtual {p1, p3}, Lorg/json/JSONArray;->optJSONObject(I)Lorg/json/JSONObject;

    move-result-object p1

    invoke-direct {p0, p1}, Lorg/garlic/client/LegacyUsers;->options(Lorg/json/JSONObject;)V

    return-void
.end method

.method synthetic lambda$showList$5$org-garlic-client-LegacyUsers(Landroid/content/DialogInterface;I)V
    .locals 0

    .line 76
    invoke-direct {p0}, Lorg/garlic/client/LegacyUsers;->create()V

    return-void
.end method

.method open()V
    .locals 1

    .line 22
    const/4 v0, 0x0

    invoke-direct {p0, v0}, Lorg/garlic/client/LegacyUsers;->request(Lorg/json/JSONObject;)V

    return-void
.end method
