.class public final synthetic Lorg/garlic/client/GarlicProxyService$0;
.super Ljava/lang/Object;
.source "D8$$SyntheticClass"

# interfaces
.implements Ljava/lang/Runnable;


# annotations
.annotation runtime Lcom/android/tools/r8/annotations/LambdaMethod;
    holder = "Lorg/garlic/client/GarlicProxyService;"
    method = "lambda$shutdownProxy$4"
    proto = "()V"
.end annotation


# instance fields
.field public final synthetic f$0:Lorg/garlic/client/GarlicProxyService;


# direct methods
.method public synthetic constructor <init>(Lorg/garlic/client/GarlicProxyService;)V
    .locals 0

    .line 0
    invoke-direct {p0}, Ljava/lang/Object;-><init>()V

    iput-object p1, p0, Lorg/garlic/client/GarlicProxyService$0;->f$0:Lorg/garlic/client/GarlicProxyService;

    return-void
.end method


# virtual methods
.method public final run()V
    .locals 1

    .line 0
    iget-object v0, p0, Lorg/garlic/client/GarlicProxyService$0;->f$0:Lorg/garlic/client/GarlicProxyService;

    invoke-virtual {v0}, Lorg/garlic/client/GarlicProxyService;->lambda$shutdownProxy$4$org-garlic-client-GarlicProxyService()V

    return-void
.end method
