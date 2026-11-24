.class public final synthetic Lorg/garlic/client/MainActivity$31;
.super Ljava/lang/Object;
.source "D8$$SyntheticClass"

# interfaces
.implements Landroid/view/View$OnClickListener;


# annotations
.annotation runtime Lcom/android/tools/r8/annotations/LambdaMethod;
    holder = "Lorg/garlic/client/MainActivity;"
    method = "lambda$subscriptionDialog$43"
    proto = "(Ljava/util/Calendar;Landroid/widget/Button;Landroid/view/View;)V"
.end annotation


# instance fields
.field public final synthetic f$0:Lorg/garlic/client/MainActivity;

.field public final synthetic f$1:Ljava/util/Calendar;

.field public final synthetic f$2:Landroid/widget/Button;


# direct methods
.method public synthetic constructor <init>(Lorg/garlic/client/MainActivity;Ljava/util/Calendar;Landroid/widget/Button;)V
    .locals 0

    .line 0
    invoke-direct {p0}, Ljava/lang/Object;-><init>()V

    iput-object p1, p0, Lorg/garlic/client/MainActivity$31;->f$0:Lorg/garlic/client/MainActivity;

    iput-object p2, p0, Lorg/garlic/client/MainActivity$31;->f$1:Ljava/util/Calendar;

    iput-object p3, p0, Lorg/garlic/client/MainActivity$31;->f$2:Landroid/widget/Button;

    return-void
.end method


# virtual methods
.method public final onClick(Landroid/view/View;)V
    .locals 3

    .line 0
    iget-object v0, p0, Lorg/garlic/client/MainActivity$31;->f$0:Lorg/garlic/client/MainActivity;

    iget-object v1, p0, Lorg/garlic/client/MainActivity$31;->f$1:Ljava/util/Calendar;

    iget-object v2, p0, Lorg/garlic/client/MainActivity$31;->f$2:Landroid/widget/Button;

    invoke-virtual {v0, v1, v2, p1}, Lorg/garlic/client/MainActivity;->lambda$subscriptionDialog$43$org-garlic-client-MainActivity(Ljava/util/Calendar;Landroid/widget/Button;Landroid/view/View;)V

    return-void
.end method
