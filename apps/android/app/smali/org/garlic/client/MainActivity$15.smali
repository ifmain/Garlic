.class public final synthetic Lorg/garlic/client/MainActivity$15;
.super Ljava/lang/Object;
.source "D8$$SyntheticClass"

# interfaces
.implements Landroid/view/View$OnClickListener;


# annotations
.annotation runtime Lcom/android/tools/r8/annotations/LambdaMethod;
    holder = "Lorg/garlic/client/MainActivity;"
    method = "lambda$showUsers$38"
    proto = "(Lorg/json/JSONObject;Landroid/view/View;)V"
.end annotation


# instance fields
.field public final synthetic f$0:Lorg/garlic/client/MainActivity;

.field public final synthetic f$1:Lorg/json/JSONObject;


# direct methods
.method public synthetic constructor <init>(Lorg/garlic/client/MainActivity;Lorg/json/JSONObject;)V
    .locals 0

    .line 0
    invoke-direct {p0}, Ljava/lang/Object;-><init>()V

    iput-object p1, p0, Lorg/garlic/client/MainActivity$15;->f$0:Lorg/garlic/client/MainActivity;

    iput-object p2, p0, Lorg/garlic/client/MainActivity$15;->f$1:Lorg/json/JSONObject;

    return-void
.end method


# virtual methods
.method public final onClick(Landroid/view/View;)V
    .locals 2

    .line 0
    iget-object v0, p0, Lorg/garlic/client/MainActivity$15;->f$0:Lorg/garlic/client/MainActivity;

    iget-object v1, p0, Lorg/garlic/client/MainActivity$15;->f$1:Lorg/json/JSONObject;

    invoke-virtual {v0, v1, p1}, Lorg/garlic/client/MainActivity;->lambda$showUsers$38$org-garlic-client-MainActivity(Lorg/json/JSONObject;Landroid/view/View;)V

    return-void
.end method
