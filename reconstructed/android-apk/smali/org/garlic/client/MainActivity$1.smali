.class Lorg/garlic/client/MainActivity$1;
.super Ljava/lang/Object;
.source "MainActivity.java"

# interfaces
.implements Ljava/lang/Runnable;


# annotations
.annotation system Ldalvik/annotation/EnclosingClass;
    value = Lorg/garlic/client/MainActivity;
.end annotation

.annotation system Ldalvik/annotation/InnerClass;
    accessFlags = 0x0
    name = null
.end annotation


# instance fields
.field final synthetic this$0:Lorg/garlic/client/MainActivity;


# direct methods
.method constructor <init>(Lorg/garlic/client/MainActivity;)V
    .locals 0
    .annotation system Ldalvik/annotation/MethodParameters;
        accessFlags = {
            0x8010
        }
        names = {
            null
        }
    .end annotation

    .line 39
    iput-object p1, p0, Lorg/garlic/client/MainActivity$1;->this$0:Lorg/garlic/client/MainActivity;

    invoke-direct {p0}, Ljava/lang/Object;-><init>()V

    return-void
.end method


# virtual methods
.method public run()V
    .locals 7

    .line 40
    iget-object v0, p0, Lorg/garlic/client/MainActivity$1;->this$0:Lorg/garlic/client/MainActivity;

    invoke-static {v0}, Lorg/garlic/client/MainActivity;->-$$Nest$fgetsubscriptionInfo(Lorg/garlic/client/MainActivity;)Landroid/widget/TextView;

    move-result-object v0

    if-eqz v0, :cond_0

    iget-object v0, p0, Lorg/garlic/client/MainActivity$1;->this$0:Lorg/garlic/client/MainActivity;

    iget-object v1, p0, Lorg/garlic/client/MainActivity$1;->this$0:Lorg/garlic/client/MainActivity;

    invoke-static {v1}, Lorg/garlic/client/MainActivity;->-$$Nest$fgetsubscriptionInfo(Lorg/garlic/client/MainActivity;)Landroid/widget/TextView;

    move-result-object v1

    iget-object v2, p0, Lorg/garlic/client/MainActivity$1;->this$0:Lorg/garlic/client/MainActivity;

    invoke-static {v2}, Lorg/garlic/client/MainActivity;->-$$Nest$msubscription(Lorg/garlic/client/MainActivity;)Ljava/lang/String;

    move-result-object v2

    invoke-static {v0, v1, v2}, Lorg/garlic/client/MainActivity;->-$$Nest$mupdateText(Lorg/garlic/client/MainActivity;Landroid/widget/TextView;Ljava/lang/String;)V

    .line 41
    :cond_0
    iget-object v0, p0, Lorg/garlic/client/MainActivity$1;->this$0:Lorg/garlic/client/MainActivity;

    invoke-static {v0}, Lorg/garlic/client/MainActivity;->-$$Nest$fgetpower(Lorg/garlic/client/MainActivity;)Landroid/widget/Button;

    move-result-object v0

    if-eqz v0, :cond_b

    .line 42
    sget-boolean v0, Lorg/garlic/client/GarlicProxyService;->isRunning:Z

    .line 43
    iget-object v1, p0, Lorg/garlic/client/MainActivity$1;->this$0:Lorg/garlic/client/MainActivity;

    sget-boolean v2, Lorg/garlic/client/GarlicProxyService;->isConnecting:Z

    invoke-static {v1, v2}, Lorg/garlic/client/MainActivity;->-$$Nest$fputstarting(Lorg/garlic/client/MainActivity;Z)V

    .line 44
    iget-object v1, p0, Lorg/garlic/client/MainActivity$1;->this$0:Lorg/garlic/client/MainActivity;

    invoke-static {v1}, Lorg/garlic/client/MainActivity;->-$$Nest$fgetrouteInfo(Lorg/garlic/client/MainActivity;)Landroid/widget/TextView;

    move-result-object v1

    const/4 v2, 0x0

    const/16 v3, 0x8

    const-string v4, ""

    if-eqz v1, :cond_3

    iget-object v1, p0, Lorg/garlic/client/MainActivity$1;->this$0:Lorg/garlic/client/MainActivity;

    iget-object v5, p0, Lorg/garlic/client/MainActivity$1;->this$0:Lorg/garlic/client/MainActivity;

    invoke-static {v5}, Lorg/garlic/client/MainActivity;->-$$Nest$fgetrouteInfo(Lorg/garlic/client/MainActivity;)Landroid/widget/TextView;

    move-result-object v5

    if-eqz v0, :cond_1

    sget-object v6, Lorg/garlic/client/GarlicProxyService;->currentRoute:Ljava/lang/String;

    goto :goto_0

    :cond_1
    move-object v6, v4

    :goto_0
    invoke-static {v1, v5, v6}, Lorg/garlic/client/MainActivity;->-$$Nest$mupdateText(Lorg/garlic/client/MainActivity;Landroid/widget/TextView;Ljava/lang/String;)V

    iget-object v1, p0, Lorg/garlic/client/MainActivity$1;->this$0:Lorg/garlic/client/MainActivity;

    invoke-static {v1}, Lorg/garlic/client/MainActivity;->-$$Nest$fgetrouteInfo(Lorg/garlic/client/MainActivity;)Landroid/widget/TextView;

    move-result-object v1

    if-eqz v0, :cond_2

    move v5, v2

    goto :goto_1

    :cond_2
    move v5, v3

    :goto_1
    invoke-virtual {v1, v5}, Landroid/widget/TextView;->setVisibility(I)V

    .line 45
    :cond_3
    iget-object v1, p0, Lorg/garlic/client/MainActivity$1;->this$0:Lorg/garlic/client/MainActivity;

    invoke-static {v1}, Lorg/garlic/client/MainActivity;->-$$Nest$fgetstatsInfo(Lorg/garlic/client/MainActivity;)Landroid/widget/TextView;

    move-result-object v1

    if-eqz v1, :cond_6

    iget-object v1, p0, Lorg/garlic/client/MainActivity$1;->this$0:Lorg/garlic/client/MainActivity;

    iget-object v5, p0, Lorg/garlic/client/MainActivity$1;->this$0:Lorg/garlic/client/MainActivity;

    invoke-static {v5}, Lorg/garlic/client/MainActivity;->-$$Nest$fgetstatsInfo(Lorg/garlic/client/MainActivity;)Landroid/widget/TextView;

    move-result-object v5

    if-eqz v0, :cond_4

    sget-object v4, Lorg/garlic/client/GarlicProxyService;->statistics:Ljava/lang/String;

    :cond_4
    invoke-static {v1, v5, v4}, Lorg/garlic/client/MainActivity;->-$$Nest$mupdateText(Lorg/garlic/client/MainActivity;Landroid/widget/TextView;Ljava/lang/String;)V

    iget-object v1, p0, Lorg/garlic/client/MainActivity$1;->this$0:Lorg/garlic/client/MainActivity;

    invoke-static {v1}, Lorg/garlic/client/MainActivity;->-$$Nest$fgetstatsInfo(Lorg/garlic/client/MainActivity;)Landroid/widget/TextView;

    move-result-object v1

    if-eqz v0, :cond_5

    iget-object v4, p0, Lorg/garlic/client/MainActivity$1;->this$0:Lorg/garlic/client/MainActivity;

    invoke-static {v4}, Lorg/garlic/client/MainActivity;->-$$Nest$fgetprefs(Lorg/garlic/client/MainActivity;)Landroid/content/SharedPreferences;

    move-result-object v4

    const-string v5, "show_statistics"

    const/4 v6, 0x1

    invoke-interface {v4, v5, v6}, Landroid/content/SharedPreferences;->getBoolean(Ljava/lang/String;Z)Z

    move-result v4

    if-eqz v4, :cond_5

    goto :goto_2

    :cond_5
    move v2, v3

    :goto_2
    invoke-virtual {v1, v2}, Landroid/widget/TextView;->setVisibility(I)V

    .line 46
    :cond_6
    iget-object v1, p0, Lorg/garlic/client/MainActivity$1;->this$0:Lorg/garlic/client/MainActivity;

    iget-object v2, p0, Lorg/garlic/client/MainActivity$1;->this$0:Lorg/garlic/client/MainActivity;

    invoke-static {v2}, Lorg/garlic/client/MainActivity;->-$$Nest$fgetpower(Lorg/garlic/client/MainActivity;)Landroid/widget/Button;

    move-result-object v2

    if-nez v0, :cond_8

    iget-object v3, p0, Lorg/garlic/client/MainActivity$1;->this$0:Lorg/garlic/client/MainActivity;

    invoke-static {v3}, Lorg/garlic/client/MainActivity;->-$$Nest$fgetstarting(Lorg/garlic/client/MainActivity;)Z

    move-result v3

    if-eqz v3, :cond_7

    goto :goto_3

    :cond_7
    const-string v3, "\u041f\u043e\u0434\u043a\u043b\u044e\u0447\u0438\u0442\u044c"

    goto :goto_4

    :cond_8
    :goto_3
    const-string v3, "\u041e\u0442\u043a\u043b\u044e\u0447\u0438\u0442\u044c"

    :goto_4
    invoke-static {v1, v2, v3}, Lorg/garlic/client/MainActivity;->-$$Nest$mupdateText(Lorg/garlic/client/MainActivity;Landroid/widget/TextView;Ljava/lang/String;)V

    .line 47
    iget-object v1, p0, Lorg/garlic/client/MainActivity$1;->this$0:Lorg/garlic/client/MainActivity;

    iget-object v2, p0, Lorg/garlic/client/MainActivity$1;->this$0:Lorg/garlic/client/MainActivity;

    invoke-static {v2}, Lorg/garlic/client/MainActivity;->-$$Nest$fgetconnection(Lorg/garlic/client/MainActivity;)Landroid/widget/TextView;

    move-result-object v2

    iget-object v3, p0, Lorg/garlic/client/MainActivity$1;->this$0:Lorg/garlic/client/MainActivity;

    invoke-static {v3}, Lorg/garlic/client/MainActivity;->-$$Nest$fgetstarting(Lorg/garlic/client/MainActivity;)Z

    move-result v3

    if-eqz v3, :cond_9

    const-string v0, "\u0413\u043e\u0442\u043e\u0432\u0438\u043c \u0437\u0430\u0449\u0438\u0449\u0451\u043d\u043d\u044b\u0439 \u043a\u0430\u043d\u0430\u043b\u2026"

    goto :goto_5

    :cond_9
    if-eqz v0, :cond_a

    const-string v0, "\u041f\u043e\u0434\u043a\u043b\u044e\u0447\u0435\u043d\u043e"

    goto :goto_5

    :cond_a
    const-string v0, "\u041d\u0435 \u043f\u043e\u0434\u043a\u043b\u044e\u0447\u0435\u043d\u043e"

    :goto_5
    invoke-static {v1, v2, v0}, Lorg/garlic/client/MainActivity;->-$$Nest$mupdateText(Lorg/garlic/client/MainActivity;Landroid/widget/TextView;Ljava/lang/String;)V

    .line 48
    sget-object v0, Lorg/garlic/client/GarlicProxyService;->lastError:Ljava/lang/String;

    if-eqz v0, :cond_b

    iget-object v0, p0, Lorg/garlic/client/MainActivity$1;->this$0:Lorg/garlic/client/MainActivity;

    iget-object v1, p0, Lorg/garlic/client/MainActivity$1;->this$0:Lorg/garlic/client/MainActivity;

    invoke-static {v1}, Lorg/garlic/client/MainActivity;->-$$Nest$fgetmessage(Lorg/garlic/client/MainActivity;)Landroid/widget/TextView;

    move-result-object v1

    sget-object v2, Lorg/garlic/client/GarlicProxyService;->lastError:Ljava/lang/String;

    invoke-static {v0, v1, v2}, Lorg/garlic/client/MainActivity;->-$$Nest$mupdateText(Lorg/garlic/client/MainActivity;Landroid/widget/TextView;Ljava/lang/String;)V

    .line 50
    :cond_b
    iget-object v0, p0, Lorg/garlic/client/MainActivity$1;->this$0:Lorg/garlic/client/MainActivity;

    invoke-static {v0}, Lorg/garlic/client/MainActivity;->-$$Nest$fgethandler(Lorg/garlic/client/MainActivity;)Landroid/os/Handler;

    move-result-object v0

    const-wide/16 v1, 0x3e8

    invoke-virtual {v0, p0, v1, v2}, Landroid/os/Handler;->postDelayed(Ljava/lang/Runnable;J)Z

    .line 51
    return-void
.end method
