.class public final synthetic Lorg/garlic/client/GarlicProxyService$3;
.super Ljava/lang/Object;
.source "D8$$SyntheticClass"

# interfaces
.implements Ljava/lang/Runnable;


# annotations
.annotation runtime Lcom/android/tools/r8/annotations/LambdaMethod;
    holder = "Lorg/garlic/client/GarlicProxyService;"
    method = "lambda$onStartCommand$2"
    proto = "(JLjava/lang/Throwable;)V"
.end annotation


# instance fields
.field public final synthetic f$0:Lorg/garlic/client/GarlicProxyService;

.field public final synthetic f$1:J

.field public final synthetic f$2:Ljava/lang/Throwable;


# direct methods
.method public synthetic constructor <init>(Lorg/garlic/client/GarlicProxyService;JLjava/lang/Throwable;)V
    .locals 0

    .line 0
    invoke-direct {p0}, Ljava/lang/Object;-><init>()V

    iput-object p1, p0, Lorg/garlic/client/GarlicProxyService$3;->f$0:Lorg/garlic/client/GarlicProxyService;

    iput-wide p2, p0, Lorg/garlic/client/GarlicProxyService$3;->f$1:J

    iput-object p4, p0, Lorg/garlic/client/GarlicProxyService$3;->f$2:Ljava/lang/Throwable;

    return-void
.end method


# virtual methods
.method public final run()V
    .locals 4

    .line 0
    iget-object v0, p0, Lorg/garlic/client/GarlicProxyService$3;->f$0:Lorg/garlic/client/GarlicProxyService;

    iget-wide v1, p0, Lorg/garlic/client/GarlicProxyService$3;->f$1:J

    iget-object v3, p0, Lorg/garlic/client/GarlicProxyService$3;->f$2:Ljava/lang/Throwable;

    invoke-virtual {v0, v1, v2, v3}, Lorg/garlic/client/GarlicProxyService;->lambda$onStartCommand$2$org-garlic-client-GarlicProxyService(JLjava/lang/Throwable;)V

    return-void
.end method
