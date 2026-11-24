.class public final synthetic Lorg/garlic/client/MainActivity$7;
.super Ljava/lang/Object;
.source "D8$$SyntheticClass"

# interfaces
.implements Landroid/view/View$OnClickListener;


# annotations
.annotation runtime Lcom/android/tools/r8/annotations/LambdaMethod;
    holder = "Lorg/garlic/client/MainActivity;"
    method = "lambda$showServers$27"
    proto = "(Landroid/view/View;)V"
.end annotation


# instance fields
.field public final synthetic f$0:Lorg/garlic/client/MainActivity;


# direct methods
.method public synthetic constructor <init>(Lorg/garlic/client/MainActivity;)V
    .locals 0

    .line 0
    invoke-direct {p0}, Ljava/lang/Object;-><init>()V

    iput-object p1, p0, Lorg/garlic/client/MainActivity$7;->f$0:Lorg/garlic/client/MainActivity;

    return-void
.end method


# virtual methods
.method public final onClick(Landroid/view/View;)V
    .locals 1

    .line 0
    iget-object v0, p0, Lorg/garlic/client/MainActivity$7;->f$0:Lorg/garlic/client/MainActivity;

    invoke-virtual {v0, p1}, Lorg/garlic/client/MainActivity;->lambda$showServers$27$org-garlic-client-MainActivity(Landroid/view/View;)V

    return-void
.end method
