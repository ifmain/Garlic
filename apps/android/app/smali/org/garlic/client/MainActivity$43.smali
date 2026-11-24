.class public final synthetic Lorg/garlic/client/MainActivity$43;
.super Ljava/lang/Object;
.source "D8$$SyntheticClass"

# interfaces
.implements Ljava/lang/Runnable;


# annotations
.annotation runtime Lcom/android/tools/r8/annotations/LambdaMethod;
    holder = "Lorg/garlic/client/MainActivity;"
    method = "lambda$showUsers$40"
    proto = "()V"
.end annotation


# instance fields
.field public final synthetic f$0:Lorg/garlic/client/MainActivity;


# direct methods
.method public synthetic constructor <init>(Lorg/garlic/client/MainActivity;)V
    .locals 0

    .line 0
    invoke-direct {p0}, Ljava/lang/Object;-><init>()V

    iput-object p1, p0, Lorg/garlic/client/MainActivity$43;->f$0:Lorg/garlic/client/MainActivity;

    return-void
.end method


# virtual methods
.method public final run()V
    .locals 1

    .line 0
    iget-object v0, p0, Lorg/garlic/client/MainActivity$43;->f$0:Lorg/garlic/client/MainActivity;

    invoke-virtual {v0}, Lorg/garlic/client/MainActivity;->lambda$showUsers$40$org-garlic-client-MainActivity()V

    return-void
.end method
