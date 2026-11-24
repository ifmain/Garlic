.class public final synthetic Lorg/garlic/client/MainActivity$19;
.super Ljava/lang/Object;
.source "D8$$SyntheticClass"

# interfaces
.implements Ljava/lang/Runnable;


# annotations
.annotation runtime Lcom/android/tools/r8/annotations/LambdaMethod;
    holder = "Lorg/garlic/client/MainActivity;"
    method = "lambda$login$4"
    proto = "(ILandroid/widget/Button;)V"
.end annotation


# instance fields
.field public final synthetic f$0:Lorg/garlic/client/MainActivity;

.field public final synthetic f$1:I

.field public final synthetic f$2:Landroid/widget/Button;


# direct methods
.method public synthetic constructor <init>(Lorg/garlic/client/MainActivity;ILandroid/widget/Button;)V
    .locals 0

    .line 0
    invoke-direct {p0}, Ljava/lang/Object;-><init>()V

    iput-object p1, p0, Lorg/garlic/client/MainActivity$19;->f$0:Lorg/garlic/client/MainActivity;

    iput p2, p0, Lorg/garlic/client/MainActivity$19;->f$1:I

    iput-object p3, p0, Lorg/garlic/client/MainActivity$19;->f$2:Landroid/widget/Button;

    return-void
.end method


# virtual methods
.method public final run()V
    .locals 3

    .line 0
    iget-object v0, p0, Lorg/garlic/client/MainActivity$19;->f$0:Lorg/garlic/client/MainActivity;

    iget v1, p0, Lorg/garlic/client/MainActivity$19;->f$1:I

    iget-object v2, p0, Lorg/garlic/client/MainActivity$19;->f$2:Landroid/widget/Button;

    invoke-virtual {v0, v1, v2}, Lorg/garlic/client/MainActivity;->lambda$login$4$org-garlic-client-MainActivity(ILandroid/widget/Button;)V

    return-void
.end method
